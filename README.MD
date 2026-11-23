# Tasmota Lights Control

Local-only Angular frontend plus Rust/Axum backend for Tasmota RGB+CT bulbs.

Runtime layout:

```text
~/tasmota-lights-control/bin/tasmota-lights-control
~/.config/tasmota-lights-control/config/config.toml
~/.config/tasmota-lights-control/data/app.sqlite3
~/.config/systemd/user/tasmota-lights-control.service
```

Backend defaults to `127.0.0.1:8080`. Put nginx in front for LAN access. Backend communicates with bulbs directly.

## Mixed colors and live status

- Mixed profiles, reset settings and on-the-fly controls accept `mixedColor`: 10 hex digits in `RR GG BB CW WW` order (cold white before warm white), optional `#`. Example: `#88C0D040BF`.
- Tasmota [11.1.0](https://github.com/arendst/Tasmota/blob/v11.1.0/tasmota/xdrv_04_light.ino#L2538-L2601) receives the hex directly. With default linked RGB/CT, `Dimmer4` preserves the RGB/white balance when setting brightness.
- SQLite migration 4 preserves existing profiles and reset settings. Legacy mixed writes with `rgbColor` and `colorTemperatureKelvin` still work; mixed reads return `mixedColor`.
- Bulb status shows `rgbColor` (`#RRGGBB`) and `colorTemperatureMired` (Tasmota's native CT setpoint, not a measurement). The API also retains `whiteChannels` (`CWWW`) and `colorTemperatureKelvin`.

## Bulb configuration

- **Settings → Bulb configuration** reads `Fade` and `Speed` from the selected bulb with [`Status 11`](https://github.com/arendst/Tasmota/blob/v11.1.0/tasmota/xdrv_04_light.ino#L1495-L1504).
- `GET /api/v1/bulbs/{bulbId}/settings` returns live `{fade, speed}`. `PUT` accepts `fade` (0 or 1) and `speed` (1–40); it sends the two [commands](https://github.com/arendst/Tasmota/blob/v11.1.0/tasmota/xdrv_04_light.ino#L3061-L3119) and checks both replies. If one fails, reload the bulb settings: the first may already have changed.
- Color, dimmer, profiles, reset and direct links never send Fade or Speed. `/api/v1/settings/reset` retains their legacy fields for API compatibility only.

## Build

From repository root:

```fish
./build.sh
```

Build scripts:

| Script | Action |
|---|---|
| `build.sh` | Builds frontend, then backend |
| `tasmota-lights-control-fe/build.sh` | Runs `npm run build` |
| `tasmota-lights-control-be/build.sh` | Runs `cargo clean`, then release build with `target-cpu=native` |

Requirements: Bash, Node/npm, Rust/Cargo. Fish is supported as interactive shell; scripts use Bash through their shebang and do not need to be sourced.

Backend release binary:

```text
tasmota-lights-control-be/target/release/tasmota-lights-control
```

Copy it manually to `~/tasmota-lights-control/bin/tasmota-lights-control`.

`target-cpu=native` optimizes for build machine CPU. Build on deployment machine, or remove that flag when distributing binary to different CPUs.

## Configure backend

Copy default config:

```fish
mkdir -p ~/.config/tasmota-lights-control/config
cp tasmota-lights-control-be/release/config.toml ~/.config/tasmota-lights-control/config/config.toml
chmod 600 ~/.config/tasmota-lights-control/config/config.toml
```

Edit `allowed_cidrs` to match bulb network. Example:

```toml
bind_address = "127.0.0.1:8080"
allowed_cidrs = ["192.168.178.0/24"]
bulb_timeout_ms = 3000
fanout_concurrency = 8
max_control_operations = 4
database_busy_timeout_ms = 5000
request_body_limit_bytes = 16384
tasmota_body_limit_bytes = 65536
shutdown_grace_ms = 10000
log_level = "info"
```

Unknown TOML keys fail startup. `database_path`, when used, must be absolute. Environment variables such as `TLC_BIND_ADDRESS` override file values.

## Install and start systemd user service

Install binary and unit:

```fish
mkdir -p ~/tasmota-lights-control/bin ~/.config/systemd/user
cp tasmota-lights-control-be/target/release/tasmota-lights-control ~/tasmota-lights-control/bin/
cp tasmota-lights-control-be/release/tasmota-lights-control.service ~/.config/systemd/user/
chmod 700 ~/tasmota-lights-control/bin/tasmota-lights-control
systemctl --user daemon-reload
systemctl --user enable --now tasmota-lights-control.service
```

Check service and logs:

```fish
systemctl --user status tasmota-lights-control.service
journalctl --user -u tasmota-lights-control.service -f
```

After binary/config updates:

```fish
systemctl --user restart tasmota-lights-control.service
```

Start service at boot before login, once as administrator:

```fish
sudo loginctl enable-linger $USER
```

User service has no `User=` or `Group=`. It runs as current user. Do not add `ProtectHome`; config and SQLite data live under home.

## Shell-script portability

Current scripts are valid on Arch Linux when Bash, npm, and Cargo are installed. Their `#!/usr/bin/env bash` shebang is independent of interactive shell, so Fish can run them with `./build.sh`.

Do not run them with `fish build.sh` or source them in Fish. Root `build.sh` expects execution from repository root; frontend/backend scripts resolve their own directories. `set -euo pipefail` is Bash syntax, not Fish syntax.

Health endpoints:

```text
GET http://127.0.0.1:8080/api/v1/health/live
GET http://127.0.0.1:8080/api/v1/health/ready
```
