//! Razer extended-matrix lighting over control interface.

use hidapi::HidDevice;

const LIGHTING_TXN: u8 = 0x1f;
const CLASS_MATRIX: u8 = 0x0f;
const CMD_SET_EFFECT: u8 = 0x02;
const CMD_SET_BRIGHTNESS: u8 = 0x04;
const ARG_VARSTORE_BACKLIGHT: [u8; 2] = [0x01, 0x05];

pub fn build_razer_cmd(txn: u8, class: u8, cmd: u8, args: &[u8]) -> [u8; 91] {
    let mut buf = [0u8; 91];
    buf[2] = txn;
    buf[6] = args.len() as u8;
    buf[7] = class;
    buf[8] = cmd;
    let n = args.len().min(80);
    buf[9..9 + n].copy_from_slice(&args[..n]);
    let mut crc = 0u8;
    for b in &buf[3..89] {
        crc ^= *b;
    }
    buf[89] = crc;
    buf
}

fn parse_color(s: &str) -> (u8, u8, u8) {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[0..2], 16),
            u8::from_str_radix(&s[2..4], 16),
            u8::from_str_radix(&s[4..6], 16),
        ) {
            return (r, g, b);
        }
    }
    (0, 255, 0)
}

pub fn apply(ctrl: &HidDevice, effect: &str, color: Option<&str>, brightness: Option<u8>) {
    let mut args = ARG_VARSTORE_BACKLIGHT.to_vec();
    match effect.to_ascii_lowercase().as_str() {
        "off" | "none" => args.extend([0x00, 0x00, 0x00, 0x00]),
        "spectrum" => args.extend([0x03, 0x00, 0x00, 0x00]),
        "wave" => args.extend([0x04, 0x01, 0x28, 0x00]),
        "breathing" => {
            let (r, g, b) = color.map(parse_color).unwrap_or((0, 255, 0));
            args.extend([0x02, 0x01, 0x00, 0x01, r, g, b]);
        }
        "reactive" => {
            let (r, g, b) = color.map(parse_color).unwrap_or((255, 0, 0));
            args.extend([0x05, 0x00, 0x02, 0x01, r, g, b]);
        }
        _ => {
            let (r, g, b) = color.map(parse_color).unwrap_or((0, 120, 255));
            args.extend([0x01, 0x00, 0x01, 0x01, r, g, b]);
        }
    }
    let cmd = build_razer_cmd(LIGHTING_TXN, CLASS_MATRIX, CMD_SET_EFFECT, &args);
    match ctrl.send_feature_report(&cmd) {
        Ok(()) => println!("[lighting] effect={effect}"),
        Err(e) => eprintln!("[lighting] effect failed: {e}"),
    }
    if let Some(br) = brightness {
        let args = [ARG_VARSTORE_BACKLIGHT[0], ARG_VARSTORE_BACKLIGHT[1], br];
        let cmd = build_razer_cmd(LIGHTING_TXN, CLASS_MATRIX, CMD_SET_BRIGHTNESS, &args);
        let _ = ctrl.send_feature_report(&cmd);
    }
}

/// Device-mode 3 unlock (enables analog stream on interface 1).
pub fn unlock_analog(ctrl: &HidDevice) -> bool {
    let cmd = build_razer_cmd(0x01, 0x00, 0x04, &[0x03, 0x00]);
    match ctrl.send_feature_report(&cmd) {
        Ok(()) => {
            println!("[hid] device-mode-3 unlock sent");
            true
        }
        Err(e) => {
            eprintln!("[hid] unlock failed: {e}");
            false
        }
    }
}
