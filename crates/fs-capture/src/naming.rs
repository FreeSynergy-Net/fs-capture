//! Safe file names. Window titles are untrusted text: they never become paths as they are.

/// Longest slug taken from a title.
pub const MAX_SLUG_LEN: usize = 40;

const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Turns any text into a short, portable name part:
/// ASCII letters and digits are kept (lower-cased), everything else becomes `_`,
/// runs of `_` collapse, edges are trimmed, length is limited, empty becomes `untitled`,
/// reserved Windows device names get a trailing `_`.
/// The result never contains `.`, `/`, `\`, `:` or any of `<>"|?*`.
pub fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut last_sep = true;
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_sep = false;
        } else if !last_sep {
            out.push('_');
            last_sep = true;
        }
        if out.len() >= MAX_SLUG_LEN {
            break;
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        return "untitled".to_string();
    }
    let mut s = trimmed.to_string();
    if RESERVED.iter().any(|r| r.eq_ignore_ascii_case(&s)) {
        s.push('_');
    }
    s
}

/// `0007_20261123T101500Z_window_notepad_notes_txt.png`
pub fn capture_file_name(nr: u32, compact_time: &str, kind: &str, title: &str) -> String {
    format!("{nr:04}_{compact_time}_{kind}_{}.png", slug(title))
}

/// Is this a file name the session itself could have written (so it is safe to
/// open inside the session folder)? Used when reading the index back.
pub fn is_session_file_name(name: &str) -> bool {
    name.len() <= 120
        && name.ends_with(".png")
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        && !name.contains("..")
}
