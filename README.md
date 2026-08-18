# tartarus-linux

**Linux analog driver for the [Razer Tartarus Pro](https://www.razer.com/gaming-keypads/razer-tartarus-pro)** (`USB 1532:0244`).

OpenRazer only exposes the device as a normal keyboard. This project unlocks **analog depth (0–255)** on each switch, maps keys and joystick axes, and ships a **local web UI** for remapping.

## Features

| Feature | Details |
|---------|---------|
| **Analog keys** | HID mode-3 unlock + continuous depth reports |
| **Binds** | Digital key *or* axis (LX/LY/RX/RY/LT/RT/…) per switch |
| **Dual-bind** | Partial press + full press (`t_on` / `t_full`) |
| **Modifier chords** | L/R Ctrl, Shift, Alt, Win + key |
| **Curves** | `linear`, `expo` (gamma), `scurve` — global or per-key |
| **Deadzones** | 5% at each end of axis travel |
| **Thumb pad** | Directional pad + thumb button, remappable |
| **Scroll wheel** | Wheel up / down / click → keys (+ optional mods) |
| **Virtual gamepad** | uinput device → `/dev/input/js*` (e.g. `js2`) |
| **Web UI** | `http://127.0.0.1:8787/` — live depths, curve graph, remap |
| **Desktop integration** | Start-menu entries + systemd user service |

## Requirements

- Linux (tested on **Nobara / Fedora 44**)
- Rust toolchain + `systemd-devel` (or `libudev-devel`)
- User in group **`input`** (for `/dev/uinput`)
- Razer Tartarus Pro connected

```bash
sudo dnf install rust cargo systemd-devel   # Fedora / Nobara
# Arch: sudo pacman -S rust systemd-libs
```

## Install

```bash
git clone https://github.com/ZidoDasZibro/tartarus-linux.git
cd tartarus-linux
bash INSTALL.sh
```

`INSTALL.sh` will:

1. `cargo build --release`
2. Install binary + helpers to `~/.local/bin/`
3. Install **Start menu** entries
4. Enable/start **systemd --user** service
5. Install udev rules (asks for sudo once)
6. Offer to add you to group `input` (re-login once if so)

### Start menu

| Entry | Action |
|-------|--------|
| **Tartarus Linux Driver** | Start the driver |
| **Tartarus Linux Config** | Open the remap UI in your **default browser** |

### CLI

```bash
tartarus-linux-start
tartarus-linux-ui      # starts driver if needed → opens http://127.0.0.1:8787/
tartarus-linux-stop
systemctl --user status tartarus-linux.service
```

### Uninstall

```bash
bash UNINSTALL.sh
```

## Quick test

```bash
# virtual joystick (name contains "Tartarus Pro Analog")
jstest /dev/input/js2

# live depths + remap
tartarus-linux-ui
```

## Configuration

File: `~/.config/tartarus-linux/config.toml`  
Also editable in the web UI (Save writes the same file; driver hot-reloads ≈1s).

### Actuation

| Option | Meaning |
|--------|---------|
| `t_on` | Depth where a key starts firing |
| `t_off` | Release threshold (hysteresis, below `t_on`) |
| `t_full` | Depth for dual-bind full action (default ~top 10%) |
| `curve` | Default axis response: `linear` / `expo` / `scurve` |
| `gamma` | Expo steepness (`>1` softer start) |

Per-key overrides: optional `t_on`, `t_off`, `t_full`, `curve`, `gamma`.

### Example key bind

```toml
# key 16 = left stick Y up, expo curve
{ mode = "axis", axis = "LY", sign = 1, curve = "expo", gamma = 1.5 }

# key 01 = Ctrl+C chord
{ mode = "key", bind = "C", lctrl = true }
```

### Thumb / wheel

```toml
[thumb]
up = "UP"
down = "DOWN"
left = "LEFT"
right = "RIGHT"
button = "F24"
wheel_up = "VOLUME_UP"
wheel_down = "VOLUME_DOWN"
wheel_click = "MENU"
```

## Architecture (short)

```
Tartarus Pro (HID)
   │  mode-3 unlock + report 0x06 (20× depth)
   ▼
tartarus-linux daemon
   ├─ process depths → keys / axes / dual-bind / curves
   ├─ thumb + wheel from physical event nodes (grab)
   ├─ uinput virtual keyboard + gamepad
   └─ embedded web UI :8787
```

**Not** an OpenRazer plugin — OpenRazer stays binary-only for this device. You can still use OpenRazer for lighting if desired; stop it if analog reports stay silent.

## Troubleshooting

| Symptom | Check |
|---------|--------|
| No analog depths | Driver log should show `device-mode-3 unlock sent`; try stopping `openrazer-daemon` |
| No `/dev/input/js*` | Membership in group `input`; udev rule; restart driver |
| Axes silent in jstest | Use the **Tartarus Pro Analog** device (often `js2`, not `js0`) |
| UI looks old | Hard-refresh browser (Ctrl+Shift+R) |
| Permission denied on uinput | `sudo usermod -aG input $USER` then re-login |

## License

[GPL-3.0-or-later](LICENSE)

Inspired by the Windows-focused [open-tartarus-driver](https://github.com/ultramonaka/open-tartarus-driver) approach; this is a from-scratch Linux port using `hidapi`, `evdev`/`uinput`, and `tiny_http`.

## Credits

- Razer Tartarus Pro owners testing on Fedora/Nobara
- Linux input subsystem (`uinput`, joydev)
