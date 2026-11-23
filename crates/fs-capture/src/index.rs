//! The list of captures in a session folder: `index.tsv`.
//!
//! Format: tab-separated text, one line per capture, first line is the header.
//! Inside a field, backslash, tab, line feed and carriage return are written as
//! `\\`, `\t`, `\n`, `\r`. Any spreadsheet or text tool can read it.

use crate::error::{CaptureError, ErrorCode, Result};
use crate::time::UtcTime;

/// File name of the list inside the session folder.
pub const INDEX_FILE: &str = "index.tsv";

const HEADER: &str = "nr\ttime_utc\tkind\ttitle\tapp\twidth\theight\tfile";

/// What was captured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A whole screen.
    Screen,
    /// One window.
    Window,
}

impl Kind {
    /// `screen` or `window`.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Screen => "screen",
            Kind::Window => "window",
        }
    }
}

/// One line of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Running number, starts at 1.
    pub nr: u32,
    /// Time of the capture (UTC).
    pub time: UtcTime,
    /// Screen or window.
    pub kind: Kind,
    /// Window title or screen name.
    pub title: String,
    /// Program name (windows only, else empty).
    pub app_name: String,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// PNG file name inside the session folder.
    pub file: String,
}

fn escape(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => o.push_str("\\\\"),
            '\t' => o.push_str("\\t"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            c => o.push(c),
        }
    }
    o
}

fn unescape(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            o.push(c);
            continue;
        }
        match it.next() {
            Some('t') => o.push('\t'),
            Some('n') => o.push('\n'),
            Some('r') => o.push('\r'),
            Some(other) => o.push(other),
            None => o.push('\\'),
        }
    }
    o
}

/// Writes the whole list as text.
pub fn format_index(entries: &[Entry]) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    for e in entries {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            e.nr,
            e.time.to_rfc3339(),
            e.kind.as_str(),
            escape(&e.title),
            escape(&e.app_name),
            e.width,
            e.height,
            e.file
        ));
    }
    out
}

fn bad(line_no: usize, why: &str) -> CaptureError {
    CaptureError::new(ErrorCode::BadIndex, format!("line {line_no}: {why}"))
}

fn parse_time(s: &str) -> Option<UtcTime> {
    // Only our own format: YYYY-MM-DDTHH:MM:SSZ
    let b = s.as_bytes();
    if b.len() != 20 || b[19] != b'Z' {
        return None;
    }
    let num = |a: usize, z: usize| s.get(a..z)?.parse::<i64>().ok();
    let (y, mo, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (h, mi, se) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    if y < 1970 || !(1..=12).contains(&mo) || !(1..=31).contains(&d) || h > 23 || mi > 59 || se > 60
    {
        return None;
    }
    // Civil date to days (Hinnant).
    let y2 = if mo <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let mp = if mo > 2 { mo - 3 } else { mo + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    let secs = days * 86_400 + h * 3600 + mi * 60 + se;
    u64::try_from(secs).ok().map(UtcTime::from_unix)
}

/// Reads the list back. Damaged lines are an error, not silently skipped.
pub fn parse_index(text: &str) -> Result<Vec<Entry>> {
    let mut lines = text.lines();
    if lines.next() != Some(HEADER) {
        return Err(bad(1, "missing or wrong header"));
    }
    let mut out = Vec::new();
    for (i, line) in lines.enumerate() {
        let n = i + 2;
        if line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 8 {
            return Err(bad(n, "expected 8 fields"));
        }
        let kind = match f[2] {
            "screen" => Kind::Screen,
            "window" => Kind::Window,
            _ => return Err(bad(n, "unknown kind")),
        };
        out.push(Entry {
            nr: f[0].parse().map_err(|_| bad(n, "bad number"))?,
            time: parse_time(f[1]).ok_or_else(|| bad(n, "bad time"))?,
            kind,
            title: unescape(f[3]),
            app_name: unescape(f[4]),
            width: f[5].parse().map_err(|_| bad(n, "bad width"))?,
            height: f[6].parse().map_err(|_| bad(n, "bad height"))?,
            file: f[7].to_string(),
        });
    }
    Ok(out)
}
