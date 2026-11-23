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
fs-capture shot --primary           # capture the primary screen into ./captures
fs-capture shot --title "Notepad"   # the one window whose title contains the text
fs-capture list
fs-capture delete 2
fs-capture zip photos.zip           # never overwrites an existing file
```

Use `--dir <folder>` for another session folder.

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

Linux build needs development packages: wayland, pipewire, libspa, xcb, dbus, clang.

## Privacy

No network, no telemetry. Output goes only into the folder you name.

## Licence

MIT OR Apache-2.0.

## Command set `capture@0.1`

`catalog/commands.json` describes the command set of this module (data only, no code): ids with
namespace (`capture.shot`), version SemVer (major = a command is removed), access per command,
and a help entry per command (en + de). Commands act only on the module's own contract and
with the rights of the person.
