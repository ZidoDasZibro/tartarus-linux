//! Two virtual devices: keyboard-only + joystick-only (games like SC see a clean pad).

use crate::keys::KeyCode;
use evdev::{
    uinput::VirtualDeviceBuilder, AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent,
    Key, UinputAbsSetup,
};
use std::io;
use std::io::Write;

pub struct VirtualPad {
    kbd: evdev::uinput::VirtualDevice,
    joy: Option<evdev::uinput::VirtualDevice>,
    pub axes_ok: bool,
}

impl VirtualPad {
    pub fn new() -> io::Result<Self> {
        // --- keyboard only ---
        let mut kbd_keys = AttributeSet::<Key>::new();
        for code in 1..=255u16 {
            kbd_keys.insert(Key::new(code));
        }
        let kbd = VirtualDeviceBuilder::new()?
            .name("Tartarus Pro Keyboard (tartarus-linux)")
            .with_keys(&kbd_keys)?
            .build()?;
        eprintln!("[uinput] keyboard device created");

        // --- joystick only (no keyboard keys) ---
        let mut joy_btns = AttributeSet::<Key>::new();
        for code in [
            0x120, 0x121, 0x122, 0x123, 0x124, // joystick btns
            0x130, 0x131, 0x133, 0x134, 0x136, 0x137, 0x13a, 0x13b, // gamepad
        ] {
            joy_btns.insert(Key::new(code));
        }

        let stick = |axis: AbsoluteAxisType| {
            UinputAbsSetup::new(axis, AbsInfo::new(0, -32767, 32767, 16, 128, 0))
        };
        let trig = |axis: AbsoluteAxisType| {
            UinputAbsSetup::new(axis, AbsInfo::new(0, 0, 255, 0, 0, 0))
        };

        let (joy, axes_ok) = match VirtualDeviceBuilder::new()?
            .name("Tartarus Pro Analog (tartarus-linux)")
            .with_keys(&joy_btns)?
            .with_absolute_axis(&stick(AbsoluteAxisType::ABS_X))?
            .with_absolute_axis(&stick(AbsoluteAxisType::ABS_Y))?
            .with_absolute_axis(&stick(AbsoluteAxisType::ABS_RX))?
            .with_absolute_axis(&stick(AbsoluteAxisType::ABS_RY))?
            .with_absolute_axis(&trig(AbsoluteAxisType::ABS_Z))?
            .with_absolute_axis(&trig(AbsoluteAxisType::ABS_RZ))?
            .build()
        {
            Ok(dev) => {
                eprintln!("[uinput] joystick device created (axes ok)");
                (Some(dev), true)
            }
            Err(e) => {
                eprintln!("[uinput] joystick FAILED ({e})");
                (None, false)
            }
        };

        let _ = std::io::stderr().flush();
        std::thread::sleep(std::time::Duration::from_millis(400));

        if let Ok(out) = std::process::Command::new("sh")
            .args([
                "-c",
                "grep -A8 'Tartarus Pro' /proc/bus/input/devices 2>/dev/null; ls -l /dev/input/js* 2>/dev/null",
            ])
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            if !s.trim().is_empty() {
                eprintln!("[uinput] kernel view:\n{s}");
            }
        }

        let mut pad = Self {
            kbd,
            joy,
            axes_ok,
        };
        // Zero all stick axes so games don't see -32767 garbage
        if pad.axes_ok {
            let _ = pad.emit_axes(&[
                (0x00, 0),
                (0x01, 0),
                (0x03, 0),
                (0x04, 0),
                (0x02, 0),
                (0x05, 0),
            ]);
        }
        Ok(pad)
    }

    pub fn key_down(&mut self, code: KeyCode) -> io::Result<()> {
        self.emit_kbd(&[
            (EventType::KEY, code, 1),
            (EventType::SYNCHRONIZATION, 0, 0),
        ])
    }

    pub fn key_up(&mut self, code: KeyCode) -> io::Result<()> {
        self.emit_kbd(&[
            (EventType::KEY, code, 0),
            (EventType::SYNCHRONIZATION, 0, 0),
        ])
    }

    /// Release key even if we think it's already up (idempotent for stuck keys).
    pub fn force_key_up(&mut self, code: KeyCode) {
        let _ = self.key_up(code);
    }

    pub fn emit_axis(&mut self, axis: u16, value: i32) -> io::Result<()> {
        self.emit_axes(&[(axis, value)])
    }

    pub fn emit_axes(&mut self, pairs: &[(u16, i32)]) -> io::Result<()> {
        if !self.axes_ok || pairs.is_empty() {
            return Ok(());
        }
        let Some(ref mut joy) = self.joy else {
            return Ok(());
        };
        let mut events: Vec<InputEvent> = pairs
            .iter()
            .map(|(a, v)| InputEvent::new_now(EventType::ABSOLUTE, *a, *v))
            .collect();
        events.push(InputEvent::new_now(EventType::SYNCHRONIZATION, 0, 0));
        match joy.emit(&events) {
            Ok(()) => Ok(()),
            Err(e) => {
                eprintln!("[uinput] joy write error: {e}");
                Err(e)
            }
        }
    }

    fn emit_kbd(&mut self, items: &[(EventType, u16, i32)]) -> io::Result<()> {
        let events: Vec<_> = items
            .iter()
            .map(|(t, c, v)| InputEvent::new_now(*t, *c, *v))
            .collect();
        self.kbd.emit(&events)
    }

    pub fn self_test_y(&mut self) {
        if !self.axes_ok {
            eprintln!("[uinput] self-test skipped");
            return;
        }
        eprintln!("[uinput] self-test: move ABS_X and ABS_Y — watch jstest on the *Analog* device");
        let _ = std::io::stderr().flush();
        for i in 0..6 {
            let v = if i % 2 == 0 { 30000 } else { -30000 };
            let _ = self.emit_axes(&[(0, v), (1, v)]);
            eprintln!("[uinput] ABS_X=ABS_Y={v}");
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
        let _ = self.emit_axes(&[(0, 0), (1, 0), (3, 0), (4, 0), (2, 0), (5, 0)]);
        eprintln!("[uinput] self-test done (axes zeroed)");
    }
}
