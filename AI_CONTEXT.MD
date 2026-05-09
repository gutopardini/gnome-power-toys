# AI Context

This repository contains **Gnome Power Toys**, a PowerToys-inspired utility app for GNOME on Wayland.

## Project Shape

- Language: Rust, edition 2024.
- UI stack: GTK 4 through the `gtk4` crate.
- Screenshot backend: XDG desktop portal through `gio::DBusConnection`.
- OCR backend: external `tesseract` command.
- Clipboard backend: external `wl-copy` command.
- Packaging: local desktop install through `Makefile`.
- App ID: `dev.gutopardini.GnomePowerToys`.

## Key Files

- `src/main.rs`: app entry point.
- `src/app.rs`: GTK application, UI layout, styling, extraction button, copy button, and `Ctrl+C` shortcut.
- `src/screenshot.rs`: portal screenshot request and response handling. Keep this portal-based; do not use GNOME Shell private screenshot APIs or X11 APIs.
- `src/ocr.rs`: Tesseract command wrapper.
- `src/clipboard.rs`: Wayland clipboard copy through `wl-copy`.
- `src/text_extractor.rs`: orchestration for screenshot capture, OCR, temporary file cleanup, and initial clipboard copy.
- `src/error.rs`: shared app error types and user-facing messages.
- `data/dev.gutopardini.GnomePowerToys.desktop`: GNOME desktop launcher.
- `data/dev.gutopardini.GnomePowerToys.metainfo.xml`: AppStream metadata.
- `assets/app-icons/gnome-power-toys-icon.svg`: official scalable app icon used by the local installer and the app header.
- `Makefile`: local `make install` target.

## Product Direction

Gnome Power Toys should behave like a utility pocketknife for GNOME/Linux. The main window is a hub with a left-side tool list and a right-side active tool surface. The current active tool is Text Extractor; Color Picker is shown as the next disabled `Soon` tool.

Future toys should be added as separate modules and surfaced in the hub rather than turning the app into a single-purpose OCR window.

## Runtime Behavior

The app opens a compact GTK hub. The sidebar lists utility toys; the content area shows the active toy.

The OCR language selector shows user-facing names:

- `Portuguese + English` maps to Tesseract `por+eng`.
- `Portuguese` maps to `por`.
- `English` maps to `eng`.

Extraction flow:

1. User clicks **Extract text**.
2. The app hides its window to keep it out of the capture.
3. The app requests an interactive screenshot through `org.freedesktop.portal.Screenshot`.
4. GNOME shows its portal screenshot UI.
5. The portal returns a screenshot file URI.
6. Tesseract OCR extracts text from the screenshot.
7. The temporary screenshot is removed.
8. The app presents the window again.
9. Extracted text is shown and copied through `wl-copy` when available.

Copy behavior:

- The result header has a copy icon button.
- `Ctrl+C` copies the latest extracted text when the app is focused.
- Copy uses `wl-copy`; if `wl-copy` is unavailable or fails, the app shows an error in the status label.

## Build And Run

Host dependencies documented in `README.md` for Fedora:

```sh
sudo dnf install gtk4-devel xdg-desktop-portal xdg-desktop-portal-gnome tesseract tesseract-langpack-por tesseract-langpack-eng wl-clipboard
```

Run locally:

```sh
cargo run
```

Install locally:

```sh
make install
```

This installs:

- `~/.local/bin/gnome-power-toys`
- `~/.local/share/applications/dev.gutopardini.GnomePowerToys.desktop`
- `~/.local/share/metainfo/dev.gutopardini.GnomePowerToys.metainfo.xml`
- `~/.local/share/icons/hicolor/scalable/apps/dev.gutopardini.GnomePowerToys.svg`

## Verification Commands

Use these for local verification:

```sh
cargo fmt -- --check
cargo test
cargo build --release --locked
```

## Development Guidance

- Preserve the XDG desktop portal capture flow for GNOME/Wayland reliability.
- Avoid GNOME Shell private D-Bus screenshot methods; they are blocked for ordinary apps.
- Keep OCR replaceable. `tesseract` is currently external, but a future backend can live behind `ocr.rs`.
- Keep UI effects lightweight. Prefer translucent surfaces, borders, and shadows over expensive live blur.
- Keep installable assets under `assets/` and desktop metadata under `data/`.
- Do not silently remove user edits. This repository may be dirty during agent work.

## Known Gaps / Risks

- The app depends on external commands for OCR and clipboard: `tesseract` and `wl-copy`.
- `Ctrl+C` is app-local, not a global Wayland shortcut.
- The app icon is a scalable SVG supplied by the project owner.
- No Flatpak manifest exists yet.
