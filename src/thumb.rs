//! Physical Tartarus: thumb pad, thumb button, scroll wheel.
//! Never grab our virtual uinput device.

use crate::config::cfg;
use crate::keys;
use crate::uinput_kb::VirtualPad;
use evdev::{Device, InputEventKind, Key, RelativeAxisType};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const HW_UP: u16 = keys::KEY_UP;
const HW_DOWN: u16 = keys::KEY_DOWN;
const HW_LEFT: u16 = keys::KEY_LEFT;
const HW_RIGHT: u16 = keys::KEY_RIGHT;
/// Physical thumb-button candidates (Razer maps vary by mode/firmware).
const HW_THUMB: &[u16] = &[
    keys::KEY_LEFTALT,
    keys::KEY_RIGHTALT,
    keys::KEY_LEFTCTRL,
    keys::KEY_RIGHTCTRL,
    keys::KEY_SPACE,
    keys::KEY_ENTER,
    125, // KEY_LEFTMETA
    126, // KEY_RIGHTMETA
    keys::KEY_F24, // if already remapped
];

fn sys_walk_is_razer(path: &PathBuf) -> bool {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let sys = PathBuf::from("/sys/class/input").join(name);
    let mut cur = sys.join("device");
    for _ in 0..12 {
        if let Ok(canon) = cur.canonicalize() {
            if canon.to_string_lossy().contains("/devices/virtual/") {
                return false;
            }
        }
        let vendor = fs::read_to_string(cur.join("idVendor")).unwrap_or_default();
        let product = fs::read_to_string(cur.join("idProduct")).unwrap_or_default();
        if vendor.trim() == "1532" && product.trim().eq_ignore_ascii_case("0244") {
            return true;
        }
        match cur.join("..").canonicalize() {
            Ok(p) => cur = p,
            Err(_) => break,
        }
    }
    false
}

fn is_physical_tartarus_kbd(path: &PathBuf) -> bool {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if !name.starts_with("event") {
        return false;
    }
    if let Ok(d) = Device::open(path) {
        let n = d.name().unwrap_or("").to_ascii_lowercase();
        if n.contains("tartarus-linux") || n.contains("analog (tartarus") {
            return false;
        }
    }
    if !sys_walk_is_razer(path) {
        return false;
    }
    if let Ok(d) = Device::open(path) {
        return d
            .supported_keys()
            .map(|k| k.contains(Key::KEY_UP))
            .unwrap_or(false);
    }
    false
}

fn is_physical_tartarus_mouse(path: &PathBuf) -> bool {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if !name.starts_with("event") {
        return false;
    }
    if let Ok(d) = Device::open(path) {
        let n = d.name().unwrap_or("").to_ascii_lowercase();
        if n.contains("tartarus-linux") {
            return false;
        }
    }
    if !sys_walk_is_razer(path) {
        return false;
    }
    // Has relative wheel or mouse axes
    if let Ok(d) = Device::open(path) {
        if let Some(rel) = d.supported_relative_axes() {
            return rel.contains(RelativeAxisType::REL_WHEEL)
                || rel.contains(RelativeAxisType::REL_HWHEEL)
                || rel.contains(RelativeAxisType::REL_Y);
        }
    }
    false
}

fn find_kbd() -> Option<PathBuf> {
    let mut found = Vec::new();
    for ent in fs::read_dir("/dev/input").ok()?.flatten() {
        let p = ent.path();
        if is_physical_tartarus_kbd(&p) {
            found.push(p);
        }
    }
    found.into_iter().next()
}

fn find_mouse() -> Option<PathBuf> {
    let mut found = Vec::new();
    for ent in fs::read_dir("/dev/input").ok()?.flatten() {
        let p = ent.path();
        if is_physical_tartarus_mouse(&p) {
            found.push(p);
        }
    }
    // Prefer node that has REL_WHEEL
    for p in &found {
        if let Ok(d) = Device::open(p) {
            if d.supported_relative_axes()
                .map(|r| r.contains(RelativeAxisType::REL_WHEEL))
                .unwrap_or(false)
            {
                return Some(p.clone());
            }
        }
    }
    found.into_iter().next()
}

fn handle_thumb_key(code: u16, pressed: bool, pad: &Arc<Mutex<VirtualPad>>) {
    let c = cfg();
    let out = if code == HW_UP {
        Some(c.thumb.up)
    } else if code == HW_DOWN {
        Some(c.thumb.down)
    } else if code == HW_LEFT {
        Some(c.thumb.left)
    } else if code == HW_RIGHT {
        Some(c.thumb.right)
    } else if HW_THUMB.contains(&code) {
        Some(c.thumb.button)
    } else {
        // Pass through other keys from grabbed kbd so nothing is "eaten" forever
        if pressed {
            eprintln!("[thumb] passthrough hw code={code}");
        }
        Some(code)
    };
    if let Some(k) = out {
        if let Ok(mut g) = pad.lock() {
            if pressed {
                let _ = g.key_down(k);
            } else {
                let _ = g.key_up(k);
            }
        }
    }
}

pub fn spawn_thumb_thread(pad: Arc<Mutex<VirtualPad>>) {
    // Keyboard / thumb pad
    let pad_k = Arc::clone(&pad);
    std::thread::spawn(move || {
        let Some(path) = find_kbd() else {
            eprintln!("[thumb] no physical Tartarus keyboard — pad remap off");
            return;
        };
        let mut dev = match Device::open(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[thumb] open {}: {e}", path.display());
                return;
            }
        };
        let n = dev.name().unwrap_or("?").to_string();
        if let Err(e) = dev.grab() {
            eprintln!("[thumb] grab failed on {n} ({e})");
        } else {
            println!("[thumb] grabbed physical {} ({})", path.display(), n);
        }
        loop {
            match dev.fetch_events() {
                Ok(events) => {
                    for ev in events {
                        if let InputEventKind::Key(key) = ev.kind() {
                            handle_thumb_key(key.0, ev.value() != 0, &pad_k);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[thumb] {e}");
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
        }
    });

    // Scroll wheel (mouse interface)
    let pad_w = Arc::clone(&pad);
    std::thread::spawn(move || {
        let Some(path) = find_mouse() else {
            eprintln!("[wheel] no Tartarus mouse/wheel node — scroll remap off");
            return;
        };
        let mut dev = match Device::open(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[wheel] open {}: {e}", path.display());
                return;
            }
        };
        let n = dev.name().unwrap_or("?").to_string();
        // Grab so OS doesn't also scroll the desktop
        if let Err(e) = dev.grab() {
            eprintln!("[wheel] grab failed ({e}) — continuing without exclusive grab");
        }
        println!("[wheel] listening {} ({})", path.display(), n);
        loop {
            match dev.fetch_events() {
                Ok(events) => {
                    for ev in events {
                        if let InputEventKind::Key(key) = ev.kind() {
                            // Middle button = wheel click (BTN_MIDDLE = 0x112 = 274)
                            let code = key.0;
                            if code == 274 || code == 272 {
                                let c = cfg();
                                let k = if code == 274 { c.thumb.wheel_click } else { c.thumb.button };
                                // only middle for wheel_click
                                if code == 274 {
                                    if let Ok(mut g) = pad_w.lock() {
                                        if ev.value() != 0 { let _ = g.key_down(c.thumb.wheel_click); }
                                        else { let _ = g.key_up(c.thumb.wheel_click); }
                                    }
                                    eprintln!("[wheel] click {}", if ev.value()!=0 {"down"} else {"up"});
                                }
                            }
                        }
                        if let InputEventKind::RelAxis(axis) = ev.kind() {
                            if axis == RelativeAxisType::REL_WHEEL {
                                let notches = ev.value();
                                if notches == 0 {
                                    continue;
                                }
                                let c = cfg();
                                let key = if notches > 0 {
                                    c.thumb.wheel_up
                                } else {
                                    c.thumb.wheel_down
                                };
                                let times = notches.unsigned_abs().min(5);
                                if let Ok(mut g) = pad_w.lock() {
                                    for _ in 0..times {
                                        let _ = g.key_down(key);
                                        let _ = g.key_up(key);
                                    }
                                }
                                eprintln!("[wheel] {} x{times}", if notches > 0 { "up" } else { "down" });
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[wheel] {e}");
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
        }
    });
}
