# fs-capture

Take screenshots of whole screens or single windows into a session folder, look at them,
delete them, and bundle everything as one ZIP. Windows and Linux.

Built with help from Claude (AI).

Two crates:

- `fs-capture` - the library. The picture source is the trait `CaptureSource`; the `xcap`
  adapter (feature `xcap`, default) is one implementation, `FakeSource` gives fixed pictures
  for tests. No type of the backend crate is part of the API.
- `fs-capture-cli` - the command line `fs-capture`.

```
fs-capture screens                  # list screens
fs-capture windows                  # list windows
fs-capture shot --title "Notepad"   # the one window whose title contains the text (default way)
fs-capture shot --window 12         # one window by number
fs-capture shot --primary --full-screen   # a WHOLE screen: only with the explicit flag
fs-capture list
fs-capture delete 2
fs-capture zip photos.zip           # never overwrites an existing file
```

Use `--dir <folder>` for another session folder.

## Whole screen only on request

A whole screen shows the contents of all programs, also private ones. So `shot --primary` and
`shot --screen <id>` are refused unless you add `--full-screen`: exit code 1, `error[E_FULL_SCREEN_NOT_CONFIRMED]`,
nothing is captured and no folder is created. With the flag a warning is printed on stderr. Windows
(`--window`, `--title`) need no flag. The rule lives in the library (type `FullScreenConsent`, default
`Denied`; `Session::add_screen` takes it), so every caller gets it, not only the CLI.

## The folder and the ZIP

The folder holds `NNNN_<UTC time>_<screen|window>_<slug>.png` and `index.tsv`. The list is
tab-separated text (nr, time_utc, kind, title, app, width, height, file): plain, open, readable
by any spreadsheet. File names never contain raw window titles (ASCII slug, max 40 characters,
no `..`, no reserved Windows names). The ZIP contains only the files named in the list, with
relative names and fixed timestamps, so bundling the same session twice gives identical bytes.

## Limits (xcap 0.9.10, read from its source)

See the notes in the report of the project: minimized windows are refused with `E_WINDOW_MINIMIZED`;
under Wayland only the screen can be captured (through the GNOME Shell or portal or wlroots
route), and windows/screens are listed through X11 (XWayland); on Windows a window is captured
by GDI `PrintWindow`.

Linux build needs development packages (Debian/Ubuntu: see `.github/workflows/check.yml`). Fedora:
`wayland-devel libxkbcommon-devel pipewire-devel libdrm-devel mesa-libgbm-devel libxcb-devel dbus-devel
libglvnd-devel clang-devel`. Tools for listing and capturing windows on X11: `xdotool xprop xwininfo`
(Fedora packages of the same names; not `xorg-x11-utils`).

## Manifest

`module.toml` says what the package offers (command set `capture@0.1`, `catalog/commands.json`), what it needs and which form shows its
data (`tiles@1`). Open and not decided here: the program name, the frontend binding and the channels of `tiles@1`; see `CHANGELOG.md`.

## Privacy

No network, no telemetry. Output goes only into the folder you name.

## Licence

MIT OR Apache-2.0.

## Command set `capture@0.1`

`catalog/commands.json` describes the command set of this module (data only, no code): ids with
namespace (`capture.shot`), version SemVer (major = a command is removed), access per command,
and a help entry per command (en + de). Commands act only on the module's own contract and
with the rights of the person.
