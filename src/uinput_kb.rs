//! Virtual joystick + keyboard via uinput (evdev 0.12).

use crate::keys::KeyCode;
use evdev::{
    uinput::VirtualDeviceBuilder, AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent,
    Key, UinputAbsSetup,
};
use std::io;
use std::io::Write;

pub struct VirtualPad {
    device: evdev::uinput::VirtualDevice,
    pub axes_ok: bool,
}

impl VirtualPad {
    pub fn new() -> io::Result<Self> {
        let mut keys = AttributeSet::<Key>::new();
        // Full keyboard
        for code in 1..=255u16 {
            keys.insert(Key::new(code));
        }
        // Classic joystick + gamepad buttons (helps joydev create /dev/input/jsN)
        for code in [
            0x120, // BTN_JOYSTICK / BTN_TRIGGER
            0x121, // BTN_THUMB
            0x122, // BTN_THUMB2
            0x123, // BTN_TOP
            0x124, // BTN_TOP2
            0x130, // BTN_SOUTH
            0x131, // BTN_EAST
            0x133, // BTN_NORTH
            0x134, // BTN_WEST
            0x136, // BTN_TL
            0x137, // BTN_TR
            0x13a, // BTN_SELECT
            0x13b, // BTN_START
        ] {
            keys.insert(Key::new(code));
        }

        match Self::build(true, &keys) {
            Ok(dev) => {
                eprintln!("[uinput] joystick device created (axes ok)");
                let _ = std::io::stderr().flush();
                std::thread::sleep(std::time::Duration::from_millis(400));
                // Print how the kernel sees us
                if let Ok(out) = std::process::Command::new("sh")
                    .args([
                        "-c",
                        "grep -A8 'Tartarus Pro Analog' /proc/bus/input/devices 2>/dev/null; ls -l /dev/input/js* 2>/dev/null; ls -l /dev/input/by-id/*Tartarus* 2>/dev/null; ls -l /dev/input/by-id/*tartarus* 2>/dev/null",
                    ])
                    .output()
                {
                    let s = String::from_utf8_lossy(&out.stdout);
                    if !s.trim().is_empty() {
                        eprintln!("[uinput] kernel view:\n{s}");
                    } else {
                        eprintln!("[uinput] WARNING: device not found in /proc/bus/input/devices yet");
                    }
                }
                Ok(Self {
                    device: dev,
                    axes_ok: true,
                })
            }
            Err(e) => {
                eprintln!("[uinput] axes FAILED ({e})");
                let dev = Self::build(false, &keys)?;
                Ok(Self {
                    device: dev,
                    axes_ok: false,
                })
            }
        }
    }

    fn build(with_axes: bool, keys: &AttributeSet<Key>) -> io::Result<evdev::uinput::VirtualDevice> {
        let mut b = VirtualDeviceBuilder::new()?
            .name("Tartarus Pro Analog (tartarus-linux)")
            .with_keys(keys)?;

        if with_axes {
            // Stick range matches typical jstest (-32767..32767)
            let stick = |axis: AbsoluteAxisType| {
                UinputAbsSetup::new(axis, AbsInfo::new(0, -32767, 32767, 0, 0, 0))
            };
            let trig = |axis: AbsoluteAxisType| {
                UinputAbsSetup::new(axis, AbsInfo::new(0, 0, 255, 0, 0, 0))
            };
            // Only 4 stick axes + 2 triggers — simpler jstest mapping
            b = b
                .with_absolute_axis(&stick(AbsoluteAxisType::ABS_X))?
                .with_absolute_axis(&stick(AbsoluteAxisType::ABS_Y))?
                .with_absolute_axis(&stick(AbsoluteAxisType::ABS_RX))?
                .with_absolute_axis(&stick(AbsoluteAxisType::ABS_RY))?
                .with_absolute_axis(&trig(AbsoluteAxisType::ABS_Z))?
                .with_absolute_axis(&trig(AbsoluteAxisType::ABS_RZ))?;
        }
        b.build()
    }

    pub fn key_down(&mut self, code: KeyCode) -> io::Result<()> {
        self.emit_raw(&[
            (EventType::KEY, code, 1),
            (EventType::SYNCHRONIZATION, 0, 0),
        ])
    }

    pub fn key_up(&mut self, code: KeyCode) -> io::Result<()> {
        self.emit_raw(&[
            (EventType::KEY, code, 0),
            (EventType::SYNCHRONIZATION, 0, 0),
        ])
    }

    pub fn emit_axis(&mut self, axis: u16, value: i32) -> io::Result<()> {
        self.emit_axes(&[(axis, value)])
    }

    pub fn emit_axes(&mut self, pairs: &[(u16, i32)]) -> io::Result<()> {
        if !self.axes_ok || pairs.is_empty() {
            return Ok(());
        }
        let mut ev = Vec::with_capacity(pairs.len() + 1);
        for (a, v) in pairs {
            ev.push((*a, *v));
        }
        // Build events
        let mut events: Vec<InputEvent> = pairs
            .iter()
            .map(|(a, v)| InputEvent::new_now(EventType::ABSOLUTE, *a, *v))
            .collect();
        events.push(InputEvent::new_now(EventType::SYNCHRONIZATION, 0, 0));
        match self.device.emit(&events) {
            Ok(()) => Ok(()),
            Err(e) => {
                eprintln!("[uinput] write error: {e}");
                Err(e)
            }
        }
    }

    fn emit_raw(&mut self, items: &[(EventType, u16, i32)]) -> io::Result<()> {
        let events: Vec<_> = items
            .iter()
            .map(|(t, c, v)| InputEvent::new_now(*t, *c, *v))
            .collect();
        self.device.emit(&events)
    }

    pub fn self_test_y(&mut self) {
        if !self.axes_ok {
            eprintln!("[uinput] self-test skipped");
            return;
        }
        eprintln!("[uinput] self-test: move ABS_X and ABS_Y for 3s — watch jstest/evtest");
        let _ = std::io::stderr().flush();
        for i in 0..6 {
            let v = if i % 2 == 0 { 30000 } else { -30000 };
            let _ = self.emit_axes(&[(0, v), (1, v)]); // X and Y
            eprintln!("[uinput] ABS_X=ABS_Y={v}");
            let _ = std::io::stderr().flush();
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        let _ = self.emit_axes(&[(0, 0), (1, 0)]);
        eprintln!("[uinput] self-test done");
    }
}
