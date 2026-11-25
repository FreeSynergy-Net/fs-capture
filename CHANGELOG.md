# Changelog

## Unreleased
- Privacy: a whole screen (`shot --primary`, `shot --screen <id>`) is captured only with the explicit flag `--full-screen`.
  Without it: exit code 1, error `E_FULL_SCREEN_NOT_CONFIRMED` (why, and how: `shot --window` / `--title`), nothing captured.
  With it: warning on stderr. Library: new type `FullScreenConsent` (default `Denied`); `Session::add_screen` takes it
  (breaking for library callers). `CaptureSource::capture_screen` now takes a `&ScreenPermit` (only `FullScreenConsent::require` makes one), so
  a raw whole-screen capture is not reachable without the check; `--full-screen` with any other command is a usage error. Tests with a spy source prove the source is not called when refused.
- `deny.toml` and a `cargo deny check licenses bans sources` step in CI (allowlist: the permissive licences of the reviewed tree).
- CI on Linux starts openbox and one xterm under xvfb, so the window list and a window capture are proven there; the error of
  the window list now says "no window manager / pure Wayland" when `_NET_CLIENT_LIST*` is missing.
- `module.toml`: module manifest (offers, needs, command set `capture@0.1`, form `tiles@1`, data package).

### Open (not decided in this repository)
- Name of the screenshot program (`captures`?) - user question 1, ENTSCHEID-form.md.
- Frontend binding (frontend only selects forms the module offers) - user question 2.
- Channels of the form `tiles@1`: the form catalog lives in fs-interfaces; until it defines them `module.toml` has no `map`.
- Contract id and the form catalog entry itself (fs-interfaces, not here).
