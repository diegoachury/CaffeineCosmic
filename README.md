<div align="center">

# ☕ Caffeine for COSMIC

**Keep your screen awake from the COSMIC™ panel.**

A lightweight indicator (in the spirit of *Amphetamine* / *Caffeine*) for the
**COSMIC** desktop on Pop!_OS that prevents the screen from turning off or the
system from suspending due to inactivity, with a single click from the system tray.

**English** · [Español](README.es.md)

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Made with Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![COSMIC](https://img.shields.io/badge/Desktop-COSMIC-7c4dff.svg)](https://system76.com/cosmic)

<img src="assets/screenshot.png" alt="Caffeine for COSMIC menu in the tray" width="340">

</div>

---

## ✨ Features

- **One-click toggle** from the COSMIC panel's Status Area.
- **Timed sessions**: 15 min, 30 min, 1 h, 2 h, 4 h — they turn off automatically when they expire.
- **Indefinite session** (stay awake until you turn it off manually).
- **Dynamic icon** that clearly shows the active / inactive state and is recolored
  to match the panel theme (symbolic icons).
- **No system dependencies**: 100% pure Rust (no GTK, no `libdbus`).
- **Tiny footprint**: a single binary, no extra daemons.

## 🧩 How does it work?

COSMIC is a Wayland desktop; the classic GNOME/X11 `caffeine` does not apply here.
Instead, this app uses COSMIC's native mechanisms:

- **Idle inhibition** → it calls `org.freedesktop.ScreenSaver.Inhibit` over D-Bus,
  an interface implemented by **`cosmic-idle`**. It stores the returned *cookie* and
  releases it with `UnInhibit` when deactivated. If the process dies, COSMIC releases
  the inhibition automatically.
- **Panel icon** → it is published as a **StatusNotifierItem** (SNI), which the
  `cosmic-applet-status-area` applet shows in the tray.

## 📋 Requirements

- **Pop!_OS / COSMIC desktop** (Wayland session).
- The **Status Area** applet enabled in the panel (included by default in COSMIC).
- `cosmic-idle` running (part of the standard COSMIC session).
- To build: **Rust 1.80+** (`cargo`).

## 📦 Installation

### COSMIC Flatpak repository

> Flathub does not accept desktop applets, so this is distributed through the
> [COSMIC Flatpak repository](https://github.com/pop-os/cosmic-flatpak)
> (submitted, pending review). Once merged, install it with:

```bash
flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo
flatpak install --user cosmic io.github.diegoachury.CaffeineCosmic
```

### From source

```bash
git clone https://github.com/diegoachury/CaffeineCosmic.git
cd CaffeineCosmic
./install.sh
```

The script builds in *release* mode and installs:

| Resource | Destination |
|---|---|
| Binary | `~/.local/bin/cosmic-caffeine` |
| Icons | `~/.local/share/icons/hicolor/scalable/{apps,status}/` |
| Launcher + AppStream | `~/.local/share/applications/` · `~/.local/share/metainfo/` |
| Autostart | `~/.config/autostart/io.github.diegoachury.CaffeineCosmic.desktop` |

Make sure `~/.local/bin` is in your `PATH`. Start it in the current session with:

```bash
cosmic-caffeine &
```

## 🖱️ Usage

- **Left click** on the ☕ icon → toggles an **indefinite** session.
- **Right click** → menu with:
  - *Keep awake (indefinite)*
  - *Activate for a time* → 15 min · 30 min · 1 h · 2 h · 4 h
  - *Deactivate* · *Quit*

The icon shows the state: **full cup** = active · **crossed-out cup** = inactive.

## 🛠️ Development

```bash
# Build
cargo build --release

# Run in the foreground (logs to stderr)
cargo run

# Checks
cargo clippy --all-targets
cargo fmt --check
```

### Project structure

```
CaffeineCosmic/
├── Cargo.toml · Cargo.lock     # Crate (ksni + zbus)
├── src/main.rs                 # Tray logic + D-Bus inhibition
├── data/                       # Installable resources (named after the App ID)
│   ├── io.github.diegoachury.CaffeineCosmic.desktop
│   ├── io.github.diegoachury.CaffeineCosmic.metainfo.xml
│   └── icons/hicolor/scalable/{apps,status}/*.svg
├── build-aux/                  # Flatpak packaging
│   ├── io.github.diegoachury.CaffeineCosmic.yml    # Manifest
│   ├── cargo-sources.json                          # Dependencies for offline builds
│   └── cosmic-flatpak/                             # Manifest for the COSMIC repo
├── assets/screenshot.png       # Screenshot (store / metainfo)
├── Makefile                    # Install (PREFIX=/app or $HOME/.local)
├── install.sh                  # Local install from source
├── LICENSE                     # GPL-3.0
└── README.md
```

## 📚 Documentation

- [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) — step-by-step packaging/publishing guide (Spanish).
- [`docs/LECCIONES-APRENDIDAS.md`](docs/LECCIONES-APRENDIDAS.md) — process history, real errors and best practices (Spanish).

## 🗺️ Roadmap

- [ ] **Publication to the COSMIC Flatpak repository** (App ID `io.github.diegoachury.CaffeineCosmic`).
  - [x] Flatpak manifest + `metainfo.xml` (AppStream) + `Makefile`.
  - [x] `cargo-sources.json` for offline builds.
  - [x] D-Bus sandbox tweak (Flatpak detection for `ksni`).
- [ ] Visible countdown in the menu for timed sessions.
- [ ] *"While an app is running"* option.
- [ ] Custom session durations.

## 🤝 Contributing

Issues and pull requests are welcome. Before submitting changes: run `cargo fmt`,
`cargo clippy`, and verify that the icon appears and toggles correctly in the
COSMIC tray.

## 💖 Donations

If this is useful to you, you can support development:

- **Bitcoin (BTC):** `bc1qdul5hesqdhsqyx7pe5cj5v3jlm8xv4mvmvznmj`
- **Ethereum (ETH):** `0xC5310aB5e06772df146217B2F75FEf156E441D2a`

Thank you! 🙏

## 📄 License

Distributed under the **GNU GPL-3.0-only** license. See [`LICENSE`](LICENSE).

## 🙏 Acknowledgments

- [`ksni`](https://crates.io/crates/ksni) — StatusNotifierItem implementation in Rust.
- [`zbus`](https://crates.io/crates/zbus) — pure-Rust D-Bus.
- The [System76](https://system76.com/) team and the **COSMIC** ecosystem.
- Inspired by *Amphetamine* (macOS) and *Caffeine* (GNOME).
