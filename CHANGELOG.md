# Changelog

## Unreleased
- `deny.toml` and a `cargo deny check licenses bans sources` step in CI (allowlist: the permissive licences of the reviewed tree).
- CI on Linux starts openbox and one xterm under xvfb, so the window list and a window capture are proven there; the error of
  the window list now says "no window manager / pure Wayland" when `_NET_CLIENT_LIST*` is missing.
- `module.toml`: module manifest (offers, needs, command set `capture@0.1`, form `tiles@1`, data package).

### Open (not decided in this repository)
- Name of the screenshot program (`captures`?) - user question 1, ENTSCHEID-form.md.
- Frontend binding (frontend only selects forms the module offers) - user question 2.
- Channels of the form `tiles@1`: the form catalog lives in fs-interfaces; until it defines them `module.toml` has no `map`.
- Contract id and the form catalog entry itself (fs-interfaces, not here).
