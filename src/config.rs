//! Config: 2 layers, key|axis, dual-bind, curves.

use crate::keys::{self, KeyCode};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub const NUM_KEYS: usize = 20;
pub const NUM_LAYERS: usize = 2;

pub const ABS_X: u16 = 0x00;
pub const ABS_Y: u16 = 0x01;
pub const ABS_Z: u16 = 0x02;
pub const ABS_RX: u16 = 0x03;
pub const ABS_RY: u16 = 0x04;
pub const ABS_RZ: u16 = 0x05;
pub const ABS_GAS: u16 = 0x09;
pub const ABS_BRAKE: u16 = 0x0a;
pub const ABS_HAT0X: u16 = 0x10;
pub const ABS_HAT0Y: u16 = 0x11;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxisId {
    LX, LY, RX, RY, LT, RT, Gas, Brake, HatX, HatY,
}

impl AxisId {
    pub fn code(self) -> u16 {
        match self {
            AxisId::LX => ABS_X,
            AxisId::LY => ABS_Y,
            AxisId::RX => ABS_RX,
            AxisId::RY => ABS_RY,
            AxisId::LT => ABS_Z,
            AxisId::RT => ABS_RZ,
            AxisId::Gas => ABS_GAS,
            AxisId::Brake => ABS_BRAKE,
            AxisId::HatX => ABS_HAT0X,
            AxisId::HatY => ABS_HAT0Y,
        }
    }
    pub fn is_trigger(self) -> bool {
        matches!(self, AxisId::LT | AxisId::RT | AxisId::Gas | AxisId::Brake)
    }
    pub fn is_hat(self) -> bool {
        matches!(self, AxisId::HatX | AxisId::HatY)
    }
    pub fn from_name(s: &str) -> Option<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "LX" | "LEFT_X" | "ABS_X" => Some(AxisId::LX),
            "LY" | "LEFT_Y" | "ABS_Y" => Some(AxisId::LY),
            "RX" | "RIGHT_X" | "ABS_RX" => Some(AxisId::RX),
            "RY" | "RIGHT_Y" | "ABS_RY" => Some(AxisId::RY),
            "LT" | "L2" | "ABS_Z" => Some(AxisId::LT),
            "RT" | "R2" | "ABS_RZ" => Some(AxisId::RT),
            "GAS" => Some(AxisId::Gas),
            "BRAKE" => Some(AxisId::Brake),
            "HAT_X" | "HAT0X" => Some(AxisId::HatX),
            "HAT_Y" | "HAT0Y" => Some(AxisId::HatY),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            AxisId::LX => "LX",
            AxisId::LY => "LY",
            AxisId::RX => "RX",
            AxisId::RY => "RY",
            AxisId::LT => "LT",
            AxisId::RT => "RT",
            AxisId::Gas => "GAS",
            AxisId::Brake => "BRAKE",
            AxisId::HatX => "HAT_X",
            AxisId::HatY => "HAT_Y",
        }
    }
}

/// Response curve for axis output (and optional key sensitivity).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Curve {
    Linear,
    /// gamma > 1 = softer start; < 1 = snappier
    Expo { gamma: f32 },
    /// Smoothstep-style S curve
    SCurve,
}

impl Curve {
    pub fn from_name(s: &str, gamma: Option<f32>) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "expo" | "exponential" => Curve::Expo {
                gamma: gamma.unwrap_or(2.0).clamp(0.2, 5.0),
            },
            "s" | "scurve" | "s-curve" | "smooth" => Curve::SCurve,
            _ => Curve::Linear,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Curve::Linear => "linear",
            Curve::Expo { .. } => "expo",
            Curve::SCurve => "scurve",
        }
    }
    pub fn gamma(self) -> f32 {
        match self {
            Curve::Expo { gamma } => gamma,
            _ => 2.0,
        }
    }
    /// Map depth 0..255 → 0.0..1.0 with 5% deadzones on each end, then curve.
    pub fn apply(self, depth: u8) -> f32 {
        // 5% floor / 5% ceiling deadzone on raw depth
        const LO: f32 = 0.05;
        const HI: f32 = 0.95;
        let x = (depth as f32 / 255.0).clamp(0.0, 1.0);
        let t = if x <= LO {
            0.0
        } else if x >= HI {
            1.0
        } else {
            (x - LO) / (HI - LO)
        };
        match self {
            Curve::Linear => t,
            Curve::Expo { gamma } => t.powf(gamma),
            Curve::SCurve => t * t * (3.0 - 2.0 * t),
        }
    }
}

/// Modifier keys held with a primary bind (distinct L/R).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub lctrl: bool,
    pub rctrl: bool,
    pub lshift: bool,
    pub rshift: bool,
    pub lalt: bool,
    pub ralt: bool,
    pub lwin: bool,
    pub rwin: bool,
}

impl Mods {
    pub fn any(self) -> bool {
        self.lctrl || self.rctrl || self.lshift || self.rshift
            || self.lalt || self.ralt || self.lwin || self.rwin
    }
    pub fn codes(self) -> Vec<KeyCode> {
        let mut v = Vec::new();
        if self.lctrl { v.push(keys::KEY_LEFTCTRL); }
        if self.rctrl { v.push(keys::KEY_RIGHTCTRL); }
        if self.lshift { v.push(keys::KEY_LEFTSHIFT); }
        if self.rshift { v.push(keys::KEY_RIGHTSHIFT); }
        if self.lalt { v.push(keys::KEY_LEFTALT); }
        if self.ralt { v.push(keys::KEY_RIGHTALT); }
        if self.lwin { v.push(keys::KEY_LEFTMETA); }
        if self.rwin { v.push(keys::KEY_RIGHTMETA); }
        v
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Bind {
    Key(KeyCode),
    Axis { axis: AxisId, sign: i8 },
    None,
}

#[derive(Clone, Copy, Debug)]
pub struct KeyBind {
    pub bind: Bind,
    pub mods: Mods,
    /// Optional second action when depth exceeds t_full (digital dual-action)
    pub bind_full: Option<KeyCode>,
    pub mods_full: Mods,
    pub t_on: Option<u8>,
    pub t_off: Option<u8>,
    pub t_full: Option<u8>,
    pub curve: Option<Curve>,
}

#[derive(Clone, Debug)]
pub struct Actuation {
    pub t_on: u8,
    pub t_off: u8,
    pub t_full: u8,
    pub curve: Curve,
}

#[derive(Clone, Debug)]
pub struct ThumbMap {
    pub up: KeyCode,
    pub down: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    /// Synthetic key for the physical thumb button (default F24 — rarely used)
    pub button: KeyCode,
    pub button_switches_layer: bool, // unused, kept for config compat
    pub wheel_up: KeyCode,
    pub wheel_down: KeyCode,
    pub wheel_click: KeyCode,
}

impl Default for ThumbMap {
    fn default() -> Self {
        Self {
            up: keys::KEY_UP,
            down: keys::KEY_DOWN,
            left: keys::KEY_LEFT,
            right: keys::KEY_RIGHT,
            button: keys::KEY_F24,
            button_switches_layer: false,
            wheel_up: keys::KEY_VOLUMEUP,
            wheel_down: keys::KEY_VOLUMEDOWN,
            wheel_click: keys::KEY_COMPOSE, // menu key as default middle-click analogue
        }
    }
}

#[derive(Clone, Debug)]
pub struct DriverConfig {
    pub layers: [[KeyBind; NUM_KEYS]; NUM_LAYERS],
    pub actuation: Actuation,
    pub thumb: ThumbMap,
}

fn default_kb(code: KeyCode) -> KeyBind {
    KeyBind {
        bind: Bind::Key(code),
        mods: Mods::default(),
        bind_full: None,
        mods_full: Mods::default(),
        t_on: None,
        t_off: None,
        t_full: None,
        curve: None,
    }
}

impl Default for DriverConfig {
    fn default() -> Self {
        let d0 = keys::default_layer();
        let d1 = keys::layer1_default();
        let mut layers = [[KeyBind {
            bind: Bind::None,
            mods: Mods::default(),
            bind_full: None,
            mods_full: Mods::default(),
            t_on: None,
            t_off: None,
            t_full: None,
            curve: None,
        }; NUM_KEYS]; NUM_LAYERS];
        for i in 0..NUM_KEYS {
            layers[0][i] = default_kb(d0[i]);
            layers[1][i] = default_kb(d1[i]);
        }
        Self {
            layers,
            actuation: Actuation {
                t_on: 100,
                t_off: 80,
                t_full: 230,  // top ~10% (depth >= 90%)
                curve: Curve::Linear,
            },
            thumb: ThumbMap::default(),
        }
    }
}

impl DriverConfig {
    pub fn thresholds(&self, layer: usize, i: usize) -> (u8, u8, u8) {
        let k = &self.layers[layer.min(1)][i];
        let t_on = k.t_on.unwrap_or(self.actuation.t_on);
        let mut t_off = k.t_off.unwrap_or(self.actuation.t_off);
        if t_off >= t_on {
            t_off = t_on.saturating_sub(20);
        }
        let mut t_full = k.t_full.unwrap_or(self.actuation.t_full);
        if t_full <= t_on {
            t_full = t_on.saturating_add(40).min(255);
        }
        (t_on, t_off, t_full)
    }

    pub fn curve_for(&self, layer: usize, i: usize) -> Curve {
        self.layers[layer.min(1)][i]
            .curve
            .unwrap_or(self.actuation.curve)
    }
}

/// depth 0-255 → axis value with curve + sign
pub fn depth_to_axis(axis: AxisId, sign: i8, depth: u8, curve: Curve) -> i32 {
    let shaped = curve.apply(depth);
    if axis.is_hat() {
        if shaped < 0.15 {
            return 0;
        }
        return sign.signum() as i32;
    }
    if axis.is_trigger() {
        return (shaped * 255.0).round().clamp(0.0, 255.0) as i32;
    }
    let mag = (shaped * 32767.0).round() as i32;
    (mag * sign.signum() as i32).clamp(-32767, 32767)
}

#[derive(Deserialize, Default)]
struct RawToml {
    #[serde(default)]
    actuation: RawActuation,
    #[serde(default)]
    analog: RawAnalog,
    #[serde(default)]
    thumb: RawThumb,
}

#[derive(Deserialize, Default)]
struct RawActuation {
    t_on: Option<u8>,
    t_off: Option<u8>,
    t_full: Option<u8>,
    curve: Option<String>,
    gamma: Option<f32>,
}

#[derive(Deserialize, Default)]
struct RawAnalog {
    #[serde(default)]
    keys: Vec<RawKey>,
    #[serde(default)]
    layer1: Vec<RawKey>,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
enum RawKey {
    Name(String),
    Full {
        #[serde(default)]
        mode: Option<String>,
        #[serde(default)]
        bind: Option<String>,
        #[serde(default)]
        bind_full: Option<String>,
        #[serde(default)]
        axis: Option<String>,
        #[serde(default)]
        sign: Option<i8>,
        #[serde(default)]
        t_on: Option<u8>,
        #[serde(default)]
        t_off: Option<u8>,
        #[serde(default)]
        t_full: Option<u8>,
        #[serde(default)]
        curve: Option<String>,
        #[serde(default)]
        gamma: Option<f32>,
        #[serde(default)]
        lctrl: Option<bool>,
        #[serde(default)]
        rctrl: Option<bool>,
        #[serde(default)]
        lshift: Option<bool>,
        #[serde(default)]
        rshift: Option<bool>,
        #[serde(default)]
        lalt: Option<bool>,
        #[serde(default)]
        ralt: Option<bool>,
        #[serde(default)]
        lwin: Option<bool>,
        #[serde(default)]
        rwin: Option<bool>,
    },
}

#[derive(Deserialize, Default)]
struct RawThumb {
    up: Option<String>,
    down: Option<String>,
    left: Option<String>,
    right: Option<String>,
    button: Option<String>,
    button_switches_layer: Option<bool>,
    wheel_up: Option<String>,
    wheel_down: Option<String>,
    wheel_click: Option<String>,
}

static CONFIG: RwLock<Option<Arc<DriverConfig>>> = RwLock::new(None);

pub fn cfg() -> Arc<DriverConfig> {
    CONFIG
        .read()
        .unwrap()
        .clone()
        .expect("config not loaded")
}

pub fn set_cfg(c: DriverConfig) {
    *CONFIG.write().unwrap() = Some(Arc::new(c));
}

pub fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("TARTARUS_CONFIG") {
        return PathBuf::from(p);
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config/tartarus-linux/config.toml");
    }
    PathBuf::from("config.toml")
}

fn parse_bind(r: &RawKey, fallback: KeyCode) -> KeyBind {
    match r {
        RawKey::Name(s) => {
            if let Some(ax) = AxisId::from_name(s) {
                KeyBind {
                    bind: Bind::Axis { axis: ax, sign: 1 },
                    mods: Mods::default(),
                    bind_full: None,
                    mods_full: Mods::default(),
                    t_on: None,
                    t_off: None,
                    t_full: None,
                    curve: None,
                }
            } else {
                KeyBind {
                    bind: Bind::Key(keys::key_from_name(s).unwrap_or(fallback)),
                    mods: Mods::default(),
                    bind_full: None,
                    mods_full: Mods::default(),
                    t_on: None,
                    t_off: None,
                    t_full: None,
                    curve: None,
                }
            }
        }
        RawKey::Full {
            mode,
            bind,
            bind_full,
            axis,
            sign,
            t_on,
            t_off,
            t_full,
            curve,
            gamma,
            lctrl,
            rctrl,
            lshift,
            rshift,
            lalt,
            ralt,
            lwin,
            rwin,
        } => {
            let mode = mode.as_deref().unwrap_or("key").to_ascii_lowercase();
            let b = if mode == "axis"
                || axis.as_ref().and_then(|a| AxisId::from_name(a)).is_some()
            {
                let ax = axis
                    .as_ref()
                    .and_then(|a| AxisId::from_name(a))
                    .or_else(|| bind.as_ref().and_then(|a| AxisId::from_name(a)))
                    .unwrap_or(AxisId::LX);
                let s = sign.unwrap_or(1);
                Bind::Axis {
                    axis: ax,
                    sign: if s >= 0 { 1 } else { -1 },
                }
            } else {
                Bind::Key(
                    bind.as_ref()
                        .and_then(|s| keys::key_from_name(s))
                        .unwrap_or(fallback),
                )
            };
            let full = bind_full
                .as_ref()
                .filter(|s| !s.trim().is_empty())
                .and_then(|s| keys::key_from_name(s));
            let curve = curve
                .as_ref()
                .map(|c| Curve::from_name(c, *gamma));
            let mods = Mods {
                lctrl: lctrl.unwrap_or(false),
                rctrl: rctrl.unwrap_or(false),
                lshift: lshift.unwrap_or(false),
                rshift: rshift.unwrap_or(false),
                lalt: lalt.unwrap_or(false),
                ralt: ralt.unwrap_or(false),
                lwin: lwin.unwrap_or(false),
                rwin: rwin.unwrap_or(false),
            };
            KeyBind {
                bind: b,
                mods,
                bind_full: full,
                mods_full: mods,
                t_on: *t_on,
                t_off: *t_off,
                t_full: *t_full,
                curve,
            }
        }
    }
}

fn parse_layer(raw: &[RawKey], fallback: [KeyCode; NUM_KEYS]) -> [KeyBind; NUM_KEYS] {
    let mut out = std::array::from_fn(|i| default_kb(fallback[i]));
    for (i, r) in raw.iter().enumerate().take(NUM_KEYS) {
        out[i] = parse_bind(r, fallback[i]);
    }
    out
}

fn parse_key_opt(s: &Option<String>, default: KeyCode) -> KeyCode {
    s.as_ref()
        .and_then(|n| keys::key_from_name(n))
        .unwrap_or(default)
}

pub fn load() -> DriverConfig {
    load_from(&config_path())
}

pub fn load_from(path: &Path) -> DriverConfig {
    let mut c = DriverConfig::default();
    let Ok(text) = std::fs::read_to_string(path) else {
        eprintln!("[config] no {} — defaults", path.display());
        return c;
    };
    let raw: RawToml = match toml::from_str(&text) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[config] parse error: {e}");
            return c;
        }
    };
    if let Some(t) = raw.actuation.t_on {
        c.actuation.t_on = t;
    }
    if let Some(t) = raw.actuation.t_off {
        c.actuation.t_off = t;
    }
    if let Some(t) = raw.actuation.t_full {
        c.actuation.t_full = t;
    }
    if let Some(ref name) = raw.actuation.curve {
        c.actuation.curve = Curve::from_name(name, raw.actuation.gamma);
    }
    if c.actuation.t_off >= c.actuation.t_on {
        c.actuation.t_off = c.actuation.t_on.saturating_sub(20);
    }
    let fb0 = keys::default_layer();
    let fb1 = keys::layer1_default();
    if !raw.analog.keys.is_empty() {
        c.layers[0] = parse_layer(&raw.analog.keys, fb0);
    }
    if !raw.analog.layer1.is_empty() {
        c.layers[1] = parse_layer(&raw.analog.layer1, fb1);
    }
    let d = ThumbMap::default();
    c.thumb.up = parse_key_opt(&raw.thumb.up, d.up);
    c.thumb.down = parse_key_opt(&raw.thumb.down, d.down);
    c.thumb.left = parse_key_opt(&raw.thumb.left, d.left);
    c.thumb.right = parse_key_opt(&raw.thumb.right, d.right);
    c.thumb.button = parse_key_opt(&raw.thumb.button, d.button);
    c.thumb.wheel_up = parse_key_opt(&raw.thumb.wheel_up, d.wheel_up);
    c.thumb.wheel_down = parse_key_opt(&raw.thumb.wheel_down, d.wheel_down);
    c.thumb.wheel_click = parse_key_opt(&raw.thumb.wheel_click, d.wheel_click);
    // layer switch removed — force off
    c.thumb.button_switches_layer = false;
    eprintln!("[config] loaded {}", path.display());
    c
}

pub fn try_reload() -> Option<DriverConfig> {
    let path = config_path();
    let text = std::fs::read_to_string(&path).ok()?;
    toml::from_str::<RawToml>(&text).ok()?;
    Some(load_from(&path))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UiKey {
    pub mode: String,
    pub bind: String,
    #[serde(default)]
    pub bind_full: String,
    pub axis: String,
    pub sign: i8,
    pub t_on: Option<u8>,
    pub t_off: Option<u8>,
    pub t_full: Option<u8>,
    #[serde(default = "default_curve_name")]
    pub curve: String,
    #[serde(default = "default_gamma")]
    pub gamma: f32,
    #[serde(default)]
    pub lctrl: bool,
    #[serde(default)]
    pub rctrl: bool,
    #[serde(default)]
    pub lshift: bool,
    #[serde(default)]
    pub rshift: bool,
    #[serde(default)]
    pub lalt: bool,
    #[serde(default)]
    pub ralt: bool,
    #[serde(default)]
    pub lwin: bool,
    #[serde(default)]
    pub rwin: bool,
}

fn default_curve_name() -> String {
    "linear".into()
}
fn default_gamma() -> f32 {
    2.0
}

#[derive(Serialize, Deserialize)]
pub struct UiPayload {
    pub keys: Vec<UiKey>,
    pub layer1: Vec<UiKey>,
    pub t_on: u8,
    pub t_off: u8,
    pub t_full: u8,
    pub curve: String,
    pub gamma: f32,
    pub thumb_up: String,
    pub thumb_down: String,
    pub thumb_left: String,
    pub thumb_right: String,
    pub thumb_button: String,
    pub button_switches_layer: bool,
    pub wheel_up: String,
    pub wheel_down: String,
    #[serde(default)]
    pub wheel_click: String,
    #[serde(default)]
    pub thumb_up_mods: Vec<String>,
    #[serde(default)]
    pub thumb_down_mods: Vec<String>,
    #[serde(default)]
    pub thumb_left_mods: Vec<String>,
    #[serde(default)]
    pub thumb_right_mods: Vec<String>,
    #[serde(default)]
    pub thumb_button_mods: Vec<String>,
    #[serde(default)]
    pub wheel_up_mods: Vec<String>,
    #[serde(default)]
    pub wheel_down_mods: Vec<String>,
    #[serde(default)]
    pub wheel_click_mods: Vec<String>,
}

fn kb_to_ui(k: &KeyBind) -> UiKey {
    match k.bind {
        Bind::Key(code) => UiKey {
            mode: "key".into(),
            bind: keys::key_to_name(code),
            bind_full: k.bind_full.map(keys::key_to_name).unwrap_or_default(),
            axis: "LX".into(),
            sign: 1,
            t_on: k.t_on,
            t_off: k.t_off,
            t_full: k.t_full,
            curve: k.curve.map(|c| c.name().into()).unwrap_or_else(|| "".into()),
            gamma: k.curve.map(|c| c.gamma()).unwrap_or(2.0),
            lctrl: k.mods.lctrl, rctrl: k.mods.rctrl,
            lshift: k.mods.lshift, rshift: k.mods.rshift,
            lalt: k.mods.lalt, ralt: k.mods.ralt,
            lwin: k.mods.lwin, rwin: k.mods.rwin,
        },
        Bind::Axis { axis, sign } => UiKey {
            mode: "axis".into(),
            bind: "".into(),
            bind_full: k.bind_full.map(keys::key_to_name).unwrap_or_default(),
            axis: axis.name().into(),
            sign,
            t_on: k.t_on,
            t_off: k.t_off,
            t_full: k.t_full,
            curve: k.curve.map(|c| c.name().into()).unwrap_or_else(|| "".into()),
            gamma: k.curve.map(|c| c.gamma()).unwrap_or(2.0),
            lctrl: false, rctrl: false, lshift: false, rshift: false,
            lalt: false, ralt: false, lwin: false, rwin: false,
        },
        Bind::None => UiKey {
            mode: "key".into(),
            bind: "".into(),
            bind_full: "".into(),
            axis: "LX".into(),
            sign: 1,
            t_on: k.t_on,
            t_off: k.t_off,
            t_full: k.t_full,
            curve: "".into(),
            gamma: 2.0,
            lctrl: false, rctrl: false, lshift: false, rshift: false,
            lalt: false, ralt: false, lwin: false, rwin: false,
        },
    }
}

fn ui_to_kb(u: &UiKey, fallback: KeyCode) -> KeyBind {
    let curve = if u.curve.trim().is_empty() {
        None
    } else {
        Some(Curve::from_name(&u.curve, Some(u.gamma)))
    };
    let bind = if u.mode == "axis" {
        let axis = AxisId::from_name(&u.axis).unwrap_or(AxisId::LX);
        Bind::Axis {
            axis,
            sign: if u.sign >= 0 { 1 } else { -1 },
        }
    } else {
        Bind::Key(keys::key_from_name(&u.bind).unwrap_or(fallback))
    };
    let bind_full = if u.bind_full.trim().is_empty() {
        None
    } else {
        keys::key_from_name(&u.bind_full)
    };
    let mods = Mods {
        lctrl: u.lctrl, rctrl: u.rctrl,
        lshift: u.lshift, rshift: u.rshift,
        lalt: u.lalt, ralt: u.ralt,
        lwin: u.lwin, rwin: u.rwin,
    };
    KeyBind {
        bind,
        mods,
        bind_full,
        mods_full: mods,
        t_on: u.t_on,
        t_off: u.t_off,
        t_full: u.t_full,
        curve,
    }
}

impl UiPayload {
    pub fn from_cfg(c: &DriverConfig) -> Self {
        Self {
            keys: c.layers[0].iter().map(kb_to_ui).collect(),
            layer1: c.layers[1].iter().map(kb_to_ui).collect(),
            t_on: c.actuation.t_on,
            t_off: c.actuation.t_off,
            t_full: c.actuation.t_full,
            curve: c.actuation.curve.name().into(),
            gamma: c.actuation.curve.gamma(),
            thumb_up: keys::key_to_name(c.thumb.up),
            thumb_down: keys::key_to_name(c.thumb.down),
            thumb_left: keys::key_to_name(c.thumb.left),
            thumb_right: keys::key_to_name(c.thumb.right),
            thumb_button: keys::key_to_name(c.thumb.button),
            button_switches_layer: false,
            wheel_up: keys::key_to_name(c.thumb.wheel_up),
            wheel_down: keys::key_to_name(c.thumb.wheel_down),
            wheel_click: keys::key_to_name(c.thumb.wheel_click),
            thumb_up_mods: vec![],
            thumb_down_mods: vec![],
            thumb_left_mods: vec![],
            thumb_right_mods: vec![],
            thumb_button_mods: vec![],
            wheel_up_mods: vec![],
            wheel_down_mods: vec![],
            wheel_click_mods: vec![],
        }
    }

    pub fn to_cfg(&self) -> DriverConfig {
        let mut c = DriverConfig::default();
        let fb0 = keys::default_layer();
        let fb1 = keys::layer1_default();
        for i in 0..NUM_KEYS {
            if let Some(u) = self.keys.get(i) {
                c.layers[0][i] = ui_to_kb(u, fb0[i]);
            }
            if let Some(u) = self.layer1.get(i) {
                c.layers[1][i] = ui_to_kb(u, fb1[i]);
            }
        }
        c.actuation.t_on = self.t_on;
        c.actuation.t_off = self.t_off.min(self.t_on.saturating_sub(1));
        c.actuation.t_full = self.t_full.max(self.t_on.saturating_add(1));
        c.actuation.curve = Curve::from_name(&self.curve, Some(self.gamma));
        c.thumb.up = keys::key_from_name(&self.thumb_up).unwrap_or(keys::KEY_UP);
        c.thumb.down = keys::key_from_name(&self.thumb_down).unwrap_or(keys::KEY_DOWN);
        c.thumb.left = keys::key_from_name(&self.thumb_left).unwrap_or(keys::KEY_LEFT);
        c.thumb.right = keys::key_from_name(&self.thumb_right).unwrap_or(keys::KEY_RIGHT);
        c.thumb.button = keys::key_from_name(&self.thumb_button).unwrap_or(keys::KEY_F24);
        c.thumb.button_switches_layer = false;
        c.thumb.wheel_up = keys::key_from_name(&self.wheel_up).unwrap_or(keys::KEY_VOLUMEUP);
        c.thumb.wheel_down = keys::key_from_name(&self.wheel_down).unwrap_or(keys::KEY_VOLUMEDOWN);
        c.thumb.wheel_click = keys::key_from_name(&self.wheel_click).unwrap_or(keys::KEY_COMPOSE);
        c
    }

    pub fn save_toml(&self) -> std::io::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        fn fmt_keys(keys: &[UiKey]) -> String {
            keys.iter()
                .map(|k| {
                    let mut p = if k.mode == "axis" {
                        format!(
                            "  {{ mode = \"axis\", axis = \"{}\", sign = {}",
                            k.axis,
                            if k.sign >= 0 { 1 } else { -1 }
                        )
                    } else {
                        format!("  {{ mode = \"key\", bind = \"{}\"", k.bind)
                    };
                    if !k.bind_full.trim().is_empty() {
                        p.push_str(&format!(", bind_full = \"{}\"", k.bind_full));
                    }
                    if let Some(v) = k.t_on {
                        p.push_str(&format!(", t_on = {v}"));
                    }
                    if let Some(v) = k.t_off {
                        p.push_str(&format!(", t_off = {v}"));
                    }
                    if let Some(v) = k.t_full {
                        p.push_str(&format!(", t_full = {v}"));
                    }
                    if !k.curve.trim().is_empty() {
                        p.push_str(&format!(", curve = \"{}\"", k.curve));
                        if k.curve == "expo" {
                            p.push_str(&format!(", gamma = {}", k.gamma));
                        }
                    }
                    if k.lctrl { p.push_str(", lctrl = true"); }
                    if k.rctrl { p.push_str(", rctrl = true"); }
                    if k.lshift { p.push_str(", lshift = true"); }
                    if k.rshift { p.push_str(", rshift = true"); }
                    if k.lalt { p.push_str(", lalt = true"); }
                    if k.ralt { p.push_str(", ralt = true"); }
                    if k.lwin { p.push_str(", lwin = true"); }
                    if k.rwin { p.push_str(", rwin = true"); }
                    p.push_str(" },");
                    p
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        let text = format!(
            r#"[actuation]
t_on = {}
t_off = {}
t_full = {}
curve = "{}"
gamma = {}

[thumb]
up = "{}"
down = "{}"
left = "{}"
right = "{}"
button = "{}"
button_switches_layer = false
wheel_up = "{}"
wheel_down = "{}"
wheel_click = "{}"

[analog]
keys = [
{}
]
layer1 = [
{}
]
"#,
            self.t_on,
            self.t_off,
            self.t_full,
            self.curve,
            self.gamma,
            self.thumb_up,
            self.thumb_down,
            self.thumb_left,
            self.thumb_right,
            self.thumb_button,
            self.wheel_up,
            self.wheel_down,
            self.wheel_click,
            fmt_keys(&self.keys),
            fmt_keys(&self.layer1),
        );
        std::fs::write(path, text)
    }
}
