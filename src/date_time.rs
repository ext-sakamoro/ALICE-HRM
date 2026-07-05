//! date time.

// ---------------------------------------------------------------------------
// Date / Time primitives (no external deps)
// ---------------------------------------------------------------------------

/// Simple date representation (year, month, day).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Date {
    #[must_use]
    pub const fn new(year: u16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }
}

/// Time of day in hours and minutes (24-hour clock).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
}

impl Time {
    #[must_use]
    pub const fn new(hour: u8, minute: u8) -> Self {
        Self { hour, minute }
    }

    /// Total minutes since midnight.
    #[must_use]
    pub const fn total_minutes(self) -> u32 {
        self.hour as u32 * 60 + self.minute as u32
    }

    /// Difference in minutes (self - other). Returns 0 if self <= other.
    #[must_use]
    pub const fn diff_minutes(self, other: Self) -> u32 {
        let a = self.total_minutes();
        let b = other.total_minutes();
        a.saturating_sub(b)
    }
}
