//! leave.

use crate::date_time::*;
use crate::employee::EmployeeId;

// Paid Leave
// ---------------------------------------------------------------------------

/// Paid leave ledger for a single employee.
#[derive(Debug, Clone)]
pub struct LeaveBalance {
    pub employee_id: EmployeeId,
    pub accrued_days: u32,
    pub used_days: u32,
}

impl LeaveBalance {
    #[must_use]
    pub const fn new(employee_id: EmployeeId) -> Self {
        Self {
            employee_id,
            accrued_days: 0,
            used_days: 0,
        }
    }

    /// Remaining paid leave days.
    #[must_use]
    pub const fn remaining(&self) -> u32 {
        self.accrued_days.saturating_sub(self.used_days)
    }

    /// Accrue additional days.
    pub const fn accrue(&mut self, days: u32) {
        self.accrued_days = self.accrued_days.saturating_add(days);
    }

    /// Use leave days. Returns false if insufficient balance.
    pub const fn use_days(&mut self, days: u32) -> bool {
        if self.remaining() >= days {
            self.used_days += days;
            true
        } else {
            false
        }
    }
}

/// Leave type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaveType {
    Annual,
    Sick,
    Personal,
    Maternity,
    Paternity,
}

/// A leave request.
#[derive(Debug, Clone)]
pub struct LeaveRequest {
    pub employee_id: EmployeeId,
    pub leave_type: LeaveType,
    pub start_date: Date,
    pub end_date: Date,
    pub days: u32,
    pub approved: bool,
}
