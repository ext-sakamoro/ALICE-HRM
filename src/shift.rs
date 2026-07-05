//! shift.

use crate::date_time::*;
use crate::employee::EmployeeId;
use std::collections::HashMap;

// Shift Scheduling
// ---------------------------------------------------------------------------

/// Day of week.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// A shift definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shift {
    pub start: Time,
    pub end: Time,
}

impl Shift {
    #[must_use]
    pub const fn new(start: Time, end: Time) -> Self {
        Self { start, end }
    }

    /// Duration in minutes.
    #[must_use]
    pub const fn duration_minutes(self) -> u32 {
        self.end.diff_minutes(self.start)
    }
}

/// Weekly schedule: maps day-of-week to an optional shift.
#[derive(Debug, Clone)]
pub struct WeeklySchedule {
    pub employee_id: EmployeeId,
    pub shifts: HashMap<DayOfWeek, Shift>,
}

impl WeeklySchedule {
    #[must_use]
    pub fn new(employee_id: EmployeeId) -> Self {
        Self {
            employee_id,
            shifts: HashMap::new(),
        }
    }

    /// Assign a shift to a day.
    pub fn assign(&mut self, day: DayOfWeek, shift: Shift) {
        self.shifts.insert(day, shift);
    }

    /// Remove a shift from a day.
    pub fn remove(&mut self, day: DayOfWeek) -> Option<Shift> {
        self.shifts.remove(&day)
    }

    /// Total scheduled minutes for the week.
    #[must_use]
    pub fn total_weekly_minutes(&self) -> u32 {
        self.shifts.values().map(|s| s.duration_minutes()).sum()
    }

    /// Number of working days in the week.
    #[must_use]
    pub fn working_days(&self) -> usize {
        self.shifts.len()
    }
}
