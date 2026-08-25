//! tartarus-linux — 2 layers, key|axis, dual-bind, curves

mod config;
mod keys;
mod lighting;
mod state;
mod thumb;
mod uinput_kb;
mod webui;

use config::{cfg, depth_to_axis, set_cfg, Bind, NUM_KEYS};
use hidapi::HidApi;
use keys::KeyCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uinput_kb::VirtualPad;

const VID: u16 = 0x1532;
const PID: u16 = 0x0244;
const ANALOG_REPORT_ID: u8 = 0x06;

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

fn main() {
    println!(
        "tartarus-linux v{} — 0.4 UI + curves + layers + axes",
        env!("CARGO_PKG_VERSION")
    );
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("webui") | Some("configui") => {
            set_cfg(config::load());
            webui::run();
            return;
        }
        Some("-h") | Some("--help") => {
            println!("Usage: tartarus-linux [webui]");
            return;
        }
        _ => {}
    }
    set_cfg(config::load());
    ctrlc::set_handler(|| SHUTDOWN.store(true, Ordering::SeqCst)).ok();
    if std::env::var("TARTARUS_NO_WEBUI").is_err() {
        std::thread::spawn(webui::run);
    }
    if let Err(e) = run_driver() {
        eprintln!("driver error: {e}");
        std::process::exit(1);
    }
}

fn open_control(api: &HidApi) -> Option<hidapi::HidDevice> {
    for info in api.device_list() {
        if info.vendor_id() == VID && info.product_id() == PID {
            if (info.usage_page() == 0x0001 && info.usage() == 0x0002)
                || info.interface_number() == 2
            {
                if let Ok(d) = info.open_device(api) {
                    return Some(d);
                }
            }
        }
    }
    None
}

fn open_analog(api: &HidApi) -> Option<hidapi::HidDevice> {
    let mut cands: Vec<_> = api
        .device_list()
        .filter(|d| d.vendor_id() == VID && d.product_id() == PID)
        .cloned()
        .collect();
    cands.sort_by_key(|d| d.interface_number());
    for info in &cands {
        if info.usage_page() == 0x0001 && info.usage() == 0x0006 {
            continue;
        }
        if let Ok(dev) = info.open_device(api) {
            let _ = dev.set_blocking_mode(false);
            println!(
                "[hid] opened if{} {}",
                info.interface_number(),
                info.path().to_string_lossy()
            );
            if info.interface_number() == 1 || info.usage() != 0x0002 {
                return Some(dev);
            }
        }
    }
    None
}

/// Held key state: partial chord + optional full chord (with its own mods).
#[derive(Clone, Copy)]
struct HeldKey {
    partial: KeyCode,
    mods: config::Mods,
    full: Option<(KeyCode, config::Mods)>,
}

fn press_chord(pad: &mut VirtualPad, mods: config::Mods, key: KeyCode) {
    for m in mods.codes() {
        let _ = pad.key_down(m);
    }
    let _ = pad.key_down(key);
}

fn release_chord(pad: &mut VirtualPad, mods: config::Mods, key: KeyCode) {
    // Key first, then mods (reverse of press). Always emit up to clear stuck keys.
    let _ = pad.key_up(key);
    for m in mods.codes().into_iter().rev() {
        let _ = pad.key_up(m);
    }
}

fn process(
    depths: &[u8; NUM_KEYS],
    key_down: &mut [Option<HeldKey>; NUM_KEYS],
    axis_last: &mut [i32; 32],
    pad: &mut VirtualPad,
) {
    let c = cfg();
    let layer = state::layer();
    let mut axis_acc = [0i32; 64];

    for i in 0..NUM_KEYS {
        let depth = depths[i];
        let (t_on, t_off, t_full) = c.thresholds(layer, i);
        let curve = c.curve_for(layer, i);
        let kb = c.layers[layer][i];

        match kb.bind {
            Bind::Key(code) => {
                if key_down[i].is_none() && depth > t_on {
                    // Partial press: only kb.mods (not mods_full)
                    press_chord(pad, kb.mods, code);
                    let mut full = None;
                    if let Some(bf) = kb.bind_full {
                        if depth > t_full {
                            // Same key as partial: only add extra mods, don't re-press key
                            if bf == code {
                                for m in kb.mods_full.codes() {
                                    let _ = pad.key_down(m);
                                }
                            } else {
                                press_chord(pad, kb.mods_full, bf);
                            }
                            full = Some((bf, kb.mods_full));
                        }
                    }
                    key_down[i] = Some(HeldKey {
                        partial: code,
                        mods: kb.mods,
                        full,
                    });
                    println!("key{:02} DOWN depth={depth} L{layer}", i + 1);
                } else if key_down[i].is_some() {
                    // Escalate to full without releasing partial
                    if let Some(held) = key_down[i] {
                        if held.full.is_none() {
                            if let Some(bf) = kb.bind_full {
                                if depth > t_full {
                                    if bf == held.partial {
                                        for m in kb.mods_full.codes() {
                                            let _ = pad.key_down(m);
                                        }
                                    } else {
                                        press_chord(pad, kb.mods_full, bf);
                                    }
                                    key_down[i] = Some(HeldKey {
                                        partial: held.partial,
                                        mods: held.mods,
                                        full: Some((bf, kb.mods_full)),
                                    });
                                    println!("key{:02} FULL depth={depth}", i + 1);
                                }
                            }
                        }
                    }
                    if depth <= t_off {
                        if let Some(held) = key_down[i].take() {
                            if let Some((fk, fm)) = held.full {
                                if fk == held.partial {
                                    // release extra full mods only, then partial chord once
                                    for m in fm.codes().into_iter().rev() {
                                        let _ = pad.key_up(m);
                                    }
                                } else {
                                    release_chord(pad, fm, fk);
                                }
                            }
                            release_chord(pad, held.mods, held.partial);
                            println!("key{:02} UP depth={depth}", i + 1);
                        }
                    }
                }
            }
            Bind::Axis { axis, sign } => {
                // Always accumulate so idle keys zero their contribution
                let v = if depth > 5 {
                    depth_to_axis(axis, sign, depth, curve)
                } else {
                    0
                };
                let code = axis.code() as usize;
                if code < 64 {
                    axis_acc[code] += v;
                }
            }
            Bind::None => {}
        }
    }

    let mut pairs: Vec<(u16, i32)> = Vec::new();
    for code in [
        config::ABS_X,
        config::ABS_Y,
        config::ABS_RX,
        config::ABS_RY,
        config::ABS_Z,
        config::ABS_RZ,
    ] {
        let mut v = axis_acc[code as usize];
        if code <= config::ABS_RY {
            v = v.clamp(-32767, 32767);
        } else {
            v = v.clamp(0, 255);
        }
        let idx = code as usize;
        if idx < axis_last.len() && axis_last[idx] != v {
            axis_last[idx] = v;
            pairs.push((code, v));
        }
    }
    // Always emit all six when any change — keeps Z/RZ from sticking
    if !pairs.is_empty() {
        // Include current values for all axes so nothing is left at -32767
        let all = [
            (config::ABS_X, axis_last[config::ABS_X as usize]),
            (config::ABS_Y, axis_last[config::ABS_Y as usize]),
            (config::ABS_RX, axis_last[config::ABS_RX as usize]),
            (config::ABS_RY, axis_last[config::ABS_RY as usize]),
            (config::ABS_Z, axis_last[config::ABS_Z as usize]),
            (config::ABS_RZ, axis_last[config::ABS_RZ as usize]),
        ];
        if let Err(e) = pad.emit_axes(&all) {
            eprintln!("[axis] emit_axes err={e}");
        }
    }
}

fn force_release(
    key_down: &mut [Option<HeldKey>; NUM_KEYS],
    pad: &mut VirtualPad,
) {
    for slot in key_down.iter_mut() {
        if let Some(held) = slot.take() {
            if let Some((fk, fm)) = held.full {
                release_chord(pad, fm, fk);
            }
            release_chord(pad, held.mods, held.partial);
        }
    }
}

fn run_driver() -> Result<(), Box<dyn std::error::Error>> {
    let api = HidApi::new()?;
    if !api
        .device_list()
        .any(|d| d.vendor_id() == VID && d.product_id() == PID)
    {
        eprintln!("Tartarus Pro not found");
        std::process::exit(1);
    }

    if let Some(c) = open_control(&api) {
        lighting::unlock_analog(&c);
    } else {
        eprintln!("[hid] WARNING: no control iface");
    }

    let analog = open_analog(&api).ok_or("no analog iface")?;
    let pad = Arc::new(Mutex::new(VirtualPad::new().map_err(|e| {
        format!("uinput failed ({e}) — need /dev/uinput access")
    })?));

    thumb::spawn_thumb_thread(Arc::clone(&pad));

    {
        let c = cfg();
        println!("[binds] layer0:");
        for i in 0..NUM_KEYS {
            match c.layers[0][i].bind {
                Bind::Axis { axis, sign } => {
                    println!("  key{:02} -> AXIS {} sign={}", i + 1, axis.name(), sign);
                }
                Bind::Key(k) if c.layers[0][i].bind_full.is_some() => {
                    println!(
                        "  key{:02} -> KEY {} + full {:?}",
                        i + 1,
                        k,
                        c.layers[0][i].bind_full
                    );
                }
                _ => {}
            }
        }
        println!(
            "[cfg] curve={} gamma={} t_on={} t_full={} layer_switch={}",
            c.actuation.curve.name(),
            c.actuation.curve.gamma(),
            c.actuation.t_on,
            c.actuation.t_full,
            c.thumb.button_switches_layer
        );
    }

    if let Ok(mut g) = pad.lock() {
        g.self_test_y();
    }

    println!("driver running — layer {}", state::layer());
    println!("web UI: http://127.0.0.1:8787/");

    let mut key_down: [Option<HeldKey>; NUM_KEYS] = [None; NUM_KEYS];
    let mut axis_last = [0i32; 32];
    let mut buf = [0u8; 64];
    let mut config_mtime = std::fs::metadata(config::config_path())
        .and_then(|m| m.modified())
        .ok();
    let mut last_check = Instant::now();
    let mut layer_prev = state::layer();

    while !SHUTDOWN.load(Ordering::SeqCst) {
        if last_check.elapsed() >= Duration::from_secs(1) {
            last_check = Instant::now();
            let mtime = std::fs::metadata(config::config_path())
                .and_then(|m| m.modified())
                .ok();
            if mtime != config_mtime {
                config_mtime = mtime;
                if let Some(n) = config::try_reload() {
                    set_cfg(n);
                    if let Ok(mut g) = pad.lock() {
                        force_release(&mut key_down, &mut g);
                    }
                    state::set_layer(0);
                    println!("config reloaded");
                }
            }
        }

        let layer = state::layer();
        if layer != layer_prev {
            if let Ok(mut g) = pad.lock() {
                force_release(&mut key_down, &mut g);
            }
            println!("[layer] -> {layer}");
            layer_prev = layer;
        }

        match analog.read(&mut buf) {
            Ok(n) if n >= 21 && buf[0] == ANALOG_REPORT_ID => {
                let mut depths = [0u8; NUM_KEYS];
                depths.copy_from_slice(&buf[1..21]);
                state::set_depths(&depths);
                if let Ok(mut g) = pad.lock() {
                    process(&depths, &mut key_down, &mut axis_last, &mut g);
                }
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_micros(500));
    }
    if let Ok(mut g) = pad.lock() {
        force_release(&mut key_down, &mut g);
    }
    Ok(())
}
