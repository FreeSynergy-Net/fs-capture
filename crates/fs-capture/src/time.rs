//! UTC time without extra crates, and the clock behind it.

use std::time::{SystemTime, UNIX_EPOCH};

/// A point in time, UTC, whole seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UtcTime {
    unix_seconds: u64,
}

impl UtcTime {
    /// From Unix seconds.
    pub fn from_unix(unix_seconds: u64) -> Self {
        UtcTime { unix_seconds }
    }

    /// Unix seconds.
    pub fn unix(self) -> u64 {
        self.unix_seconds
    }

    /// (year, month, day, hour, minute, second).
    pub fn parts(self) -> (i64, u32, u32, u32, u32, u32) {
        let days = (self.unix_seconds / 86_400) as i64;
        let rem = (self.unix_seconds % 86_400) as u32;
        // Days to civil date (H. Hinnant's algorithm), valid for all dates here.
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        let y = yoe + era * 400 + i64::from(m <= 2);
        (y, m, d, rem / 3600, (rem % 3600) / 60, rem % 60)
    }

    /// `2026-11-23T10:15:00Z`
    pub fn to_rfc3339(self) -> String {
        let (y, mo, d, h, mi, s) = self.parts();
        format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
    }

    /// `20261123T101500Z` (usable in file names).
    pub fn to_compact(self) -> String {
        let (y, mo, d, h, mi, s) = self.parts();
        format!("{y:04}{mo:02}{d:02}T{h:02}{mi:02}{s:02}Z")
    }
}

/// Where the session gets "now" from; tests use a fixed clock.
pub trait Clock {
    /// The current time.
    fn now(&self) -> UtcTime;
}

/// The system clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> UtcTime {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        UtcTime::from_unix(secs)
    }
}

/// A clock that always returns the same time.
#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub UtcTime);

impl Clock for FixedClock {
    fn now(&self) -> UtcTime {
        self.0
    }
}
