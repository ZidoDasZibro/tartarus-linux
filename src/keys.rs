//! Linux keycodes (linux/input-event-codes.h) — full PC-style set for apps.

pub type KeyCode = u16;

pub const KEY_ESC: KeyCode = 1;
pub const KEY_1: KeyCode = 2;
pub const KEY_2: KeyCode = 3;
pub const KEY_3: KeyCode = 4;
pub const KEY_4: KeyCode = 5;
pub const KEY_5: KeyCode = 6;
pub const KEY_6: KeyCode = 7;
pub const KEY_7: KeyCode = 8;
pub const KEY_8: KeyCode = 9;
pub const KEY_9: KeyCode = 10;
pub const KEY_0: KeyCode = 11;
pub const KEY_MINUS: KeyCode = 12;
pub const KEY_EQUAL: KeyCode = 13;
pub const KEY_BACKSPACE: KeyCode = 14;
pub const KEY_TAB: KeyCode = 15;
pub const KEY_Q: KeyCode = 16;
pub const KEY_W: KeyCode = 17;
pub const KEY_E: KeyCode = 18;
pub const KEY_R: KeyCode = 19;
pub const KEY_T: KeyCode = 20;
pub const KEY_Y: KeyCode = 21;
pub const KEY_U: KeyCode = 22;
pub const KEY_I: KeyCode = 23;
pub const KEY_O: KeyCode = 24;
pub const KEY_P: KeyCode = 25;
pub const KEY_LEFTBRACE: KeyCode = 26;
pub const KEY_RIGHTBRACE: KeyCode = 27;
pub const KEY_ENTER: KeyCode = 28;
pub const KEY_LEFTCTRL: KeyCode = 29;
pub const KEY_A: KeyCode = 30;
pub const KEY_S: KeyCode = 31;
pub const KEY_D: KeyCode = 32;
pub const KEY_F: KeyCode = 33;
pub const KEY_G: KeyCode = 34;
pub const KEY_H: KeyCode = 35;
pub const KEY_J: KeyCode = 36;
pub const KEY_K: KeyCode = 37;
pub const KEY_L: KeyCode = 38;
pub const KEY_SEMICOLON: KeyCode = 39;
pub const KEY_APOSTROPHE: KeyCode = 40;
pub const KEY_GRAVE: KeyCode = 41;
pub const KEY_LEFTSHIFT: KeyCode = 42;
pub const KEY_BACKSLASH: KeyCode = 43;
pub const KEY_Z: KeyCode = 44;
pub const KEY_X: KeyCode = 45;
pub const KEY_C: KeyCode = 46;
pub const KEY_V: KeyCode = 47;
pub const KEY_B: KeyCode = 48;
pub const KEY_N: KeyCode = 49;
pub const KEY_M: KeyCode = 50;
pub const KEY_COMMA: KeyCode = 51;
pub const KEY_DOT: KeyCode = 52;
pub const KEY_SLASH: KeyCode = 53;
pub const KEY_RIGHTSHIFT: KeyCode = 54;
pub const KEY_KPASTERISK: KeyCode = 55;
pub const KEY_LEFTALT: KeyCode = 56;
pub const KEY_SPACE: KeyCode = 57;
pub const KEY_CAPSLOCK: KeyCode = 58;
pub const KEY_F1: KeyCode = 59;
pub const KEY_F2: KeyCode = 60;
pub const KEY_F3: KeyCode = 61;
pub const KEY_F4: KeyCode = 62;
pub const KEY_F5: KeyCode = 63;
pub const KEY_F6: KeyCode = 64;
pub const KEY_F7: KeyCode = 65;
pub const KEY_F8: KeyCode = 66;
pub const KEY_F9: KeyCode = 67;
pub const KEY_F10: KeyCode = 68;
pub const KEY_NUMLOCK: KeyCode = 69;
pub const KEY_SCROLLLOCK: KeyCode = 70;
pub const KEY_KP7: KeyCode = 71;
pub const KEY_KP8: KeyCode = 72;
pub const KEY_KP9: KeyCode = 73;
pub const KEY_KPMINUS: KeyCode = 74;
pub const KEY_KP4: KeyCode = 75;
pub const KEY_KP5: KeyCode = 76;
pub const KEY_KP6: KeyCode = 77;
pub const KEY_KPPLUS: KeyCode = 78;
pub const KEY_KP1: KeyCode = 79;
pub const KEY_KP2: KeyCode = 80;
pub const KEY_KP3: KeyCode = 81;
pub const KEY_KP0: KeyCode = 82;
pub const KEY_KPDOT: KeyCode = 83;
pub const KEY_F11: KeyCode = 87;
pub const KEY_F12: KeyCode = 88;
pub const KEY_KPENTER: KeyCode = 96;
pub const KEY_RIGHTCTRL: KeyCode = 97;
pub const KEY_KPSLASH: KeyCode = 98;
pub const KEY_RIGHTALT: KeyCode = 100;
pub const KEY_HOME: KeyCode = 102;
pub const KEY_UP: KeyCode = 103;
pub const KEY_PAGEUP: KeyCode = 104;
pub const KEY_LEFT: KeyCode = 105;
pub const KEY_RIGHT: KeyCode = 106;
pub const KEY_END: KeyCode = 107;
pub const KEY_DOWN: KeyCode = 108;
pub const KEY_PAGEDOWN: KeyCode = 109;
pub const KEY_INSERT: KeyCode = 110;
pub const KEY_DELETE: KeyCode = 111;
pub const KEY_MUTE: KeyCode = 113;
pub const KEY_VOLUMEDOWN: KeyCode = 114;
pub const KEY_VOLUMEUP: KeyCode = 115;
pub const KEY_LEFTMETA: KeyCode = 125; // Windows / Super
pub const KEY_RIGHTMETA: KeyCode = 126;
pub const KEY_COMPOSE: KeyCode = 127; // Menu
pub const KEY_NEXTSONG: KeyCode = 163;
pub const KEY_PLAYPAUSE: KeyCode = 164;
pub const KEY_PREVIOUSSONG: KeyCode = 165;
pub const KEY_STOPCD: KeyCode = 166;
pub const KEY_F13: KeyCode = 183;
pub const KEY_F14: KeyCode = 184;
pub const KEY_F15: KeyCode = 185;
pub const KEY_F16: KeyCode = 186;
pub const KEY_F17: KeyCode = 187;
pub const KEY_F18: KeyCode = 188;
pub const KEY_F19: KeyCode = 189;
pub const KEY_F20: KeyCode = 190;
pub const KEY_F21: KeyCode = 191;
pub const KEY_F22: KeyCode = 192;
pub const KEY_F23: KeyCode = 193;
pub const KEY_F24: KeyCode = 194;
pub const KEY_PRINT: KeyCode = 210;
pub const KEY_PAUSE: KeyCode = 119;

const TABLE: &[(&str, KeyCode)] = &[
    ("ESC", KEY_ESC), ("ESCAPE", KEY_ESC),
    ("1", KEY_1), ("2", KEY_2), ("3", KEY_3), ("4", KEY_4), ("5", KEY_5),
    ("6", KEY_6), ("7", KEY_7), ("8", KEY_8), ("9", KEY_9), ("0", KEY_0),
    ("MINUS", KEY_MINUS), ("EQUAL", KEY_EQUAL), ("BACKSPACE", KEY_BACKSPACE),
    ("TAB", KEY_TAB),
    ("Q", KEY_Q), ("W", KEY_W), ("E", KEY_E), ("R", KEY_R), ("T", KEY_T),
    ("Y", KEY_Y), ("U", KEY_U), ("I", KEY_I), ("O", KEY_O), ("P", KEY_P),
    ("LEFTBRACE", KEY_LEFTBRACE), ("RIGHTBRACE", KEY_RIGHTBRACE),
    ("ENTER", KEY_ENTER),
    ("LCTRL", KEY_LEFTCTRL), ("RCTRL", KEY_RIGHTCTRL),
    ("A", KEY_A), ("S", KEY_S), ("D", KEY_D), ("F", KEY_F), ("G", KEY_G),
    ("H", KEY_H), ("J", KEY_J), ("K", KEY_K), ("L", KEY_L),
    ("SEMICOLON", KEY_SEMICOLON), ("APOSTROPHE", KEY_APOSTROPHE), ("GRAVE", KEY_GRAVE),
    ("LSHIFT", KEY_LEFTSHIFT), ("RSHIFT", KEY_RIGHTSHIFT),
    ("BACKSLASH", KEY_BACKSLASH),
    ("Z", KEY_Z), ("X", KEY_X), ("C", KEY_C), ("V", KEY_V), ("B", KEY_B),
    ("N", KEY_N), ("M", KEY_M),
    ("COMMA", KEY_COMMA), ("DOT", KEY_DOT), ("SLASH", KEY_SLASH),
    ("LALT", KEY_LEFTALT), ("RALT", KEY_RIGHTALT),
    ("LWIN", KEY_LEFTMETA), ("RWIN", KEY_RIGHTMETA), ("LMETA", KEY_LEFTMETA), ("RMETA", KEY_RIGHTMETA),
    ("MENU", KEY_COMPOSE), ("COMPOSE", KEY_COMPOSE),
    ("SPACE", KEY_SPACE), ("CAPSLOCK", KEY_CAPSLOCK),
    ("F1", KEY_F1), ("F2", KEY_F2), ("F3", KEY_F3), ("F4", KEY_F4),
    ("F5", KEY_F5), ("F6", KEY_F6), ("F7", KEY_F7), ("F8", KEY_F8),
    ("F9", KEY_F9), ("F10", KEY_F10), ("F11", KEY_F11), ("F12", KEY_F12),
    ("F13", KEY_F13), ("F14", KEY_F14), ("F15", KEY_F15), ("F16", KEY_F16),
    ("F17", KEY_F17), ("F18", KEY_F18), ("F19", KEY_F19), ("F20", KEY_F20),
    ("F21", KEY_F21), ("F22", KEY_F22), ("F23", KEY_F23), ("F24", KEY_F24),
    ("NUMLOCK", KEY_NUMLOCK), ("SCROLLLOCK", KEY_SCROLLLOCK),
    ("KP0", KEY_KP0), ("KP1", KEY_KP1), ("KP2", KEY_KP2), ("KP3", KEY_KP3),
    ("KP4", KEY_KP4), ("KP5", KEY_KP5), ("KP6", KEY_KP6), ("KP7", KEY_KP7),
    ("KP8", KEY_KP8), ("KP9", KEY_KP9), ("KPDOT", KEY_KPDOT),
    ("KPENTER", KEY_KPENTER), ("KPPLUS", KEY_KPPLUS), ("KPMINUS", KEY_KPMINUS),
    ("KPASTERISK", KEY_KPASTERISK), ("KPSLASH", KEY_KPSLASH),
    ("HOME", KEY_HOME), ("END", KEY_END), ("PAGEUP", KEY_PAGEUP), ("PAGEDOWN", KEY_PAGEDOWN),
    ("INSERT", KEY_INSERT), ("DELETE", KEY_DELETE),
    ("UP", KEY_UP), ("DOWN", KEY_DOWN), ("LEFT", KEY_LEFT), ("RIGHT", KEY_RIGHT),
    ("PRINT", KEY_PRINT), ("PAUSE", KEY_PAUSE),
    ("VOLUME_MUTE", KEY_MUTE), ("VOLUME_DOWN", KEY_VOLUMEDOWN), ("VOLUME_UP", KEY_VOLUMEUP),
    ("MEDIA_PLAY_PAUSE", KEY_PLAYPAUSE), ("MEDIA_STOP", KEY_STOPCD),
    ("MEDIA_PREV", KEY_PREVIOUSSONG), ("MEDIA_NEXT", KEY_NEXTSONG),
];

pub fn key_from_name(name: &str) -> Option<KeyCode> {
    let u = name.trim().to_ascii_uppercase().replace(' ', "_");
    TABLE.iter().find(|(n, _)| *n == u).map(|(_, k)| *k)
}

pub fn key_to_name(code: KeyCode) -> String {
    TABLE
        .iter()
        .find(|(_, k)| *k == code)
        .map(|(n, _)| n.to_string())
        .unwrap_or_else(|| format!("KEY_{code}"))
}

pub fn default_layer() -> [KeyCode; 20] {
    [
        KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9, KEY_0,
        KEY_A, KEY_B, KEY_C, KEY_D, KEY_E, KEY_F, KEY_G, KEY_H, KEY_I, KEY_J,
    ]
}

pub fn layer1_default() -> [KeyCode; 20] {
    [
        KEY_F1, KEY_F2, KEY_F3, KEY_F4, KEY_F5, KEY_F6, KEY_F7, KEY_F8, KEY_F9, KEY_F10,
        KEY_F11, KEY_F12, KEY_HOME, KEY_END, KEY_PAGEUP, KEY_PAGEDOWN, KEY_INSERT, KEY_DELETE,
        KEY_UP, KEY_DOWN,
    ]
}

pub fn all_key_names() -> Vec<&'static str> {
    TABLE.iter().map(|(n, _)| *n).filter(|n| *n != "ESCAPE").collect()
}
