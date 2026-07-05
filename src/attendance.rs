//! attendance.

use crate::date_time::*;
use crate::employee::EmployeeId;

// Attendance
// ---------------------------------------------------------------------------

/// A single attendance record for one day.
#[derive(Debug, Clone)]
pub struct AttendanceRecord {
    pub employee_id: EmployeeId,
    pub date: Date,
    pub clock_in: Option<Time>,
    pub clock_out: Option<Time>,
}

impl AttendanceRecord {
    #[must_use]
    pub const fn new(employee_id: EmployeeId, date: Date) -> Self {
        Self {
            employee_id,
            date,
            clock_in: None,
            clock_out: None,
        }
    }

    /// Worked minutes for the day.
    #[must_use]
    pub const fn worked_minutes(&self) -> u32 {
        match (self.clock_in, self.clock_out) {
            (Some(i), Some(o)) => o.diff_minutes(i),
            _ => 0,
        }
    }

    /// Overtime minutes beyond `standard_minutes`.
    #[must_use]
    pub const fn overtime_minutes(&self, standard_minutes: u32) -> u32 {
        self.worked_minutes().saturating_sub(standard_minutes)
    }
}
