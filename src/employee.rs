//! employee.

use crate::date_time::Date;

// Employee
// ---------------------------------------------------------------------------

/// Employment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmploymentStatus {
    Active,
    OnLeave,
    Terminated,
}

/// Department identifier.
pub type DepartmentId = u32;

/// Unique employee identifier.
pub type EmployeeId = u64;

/// Core employee record.
#[derive(Debug, Clone)]
pub struct Employee {
    pub id: EmployeeId,
    pub name: String,
    pub department_id: DepartmentId,
    pub status: EmploymentStatus,
    pub hire_date: Date,
    pub base_monthly_pay: u64,
}

impl Employee {
    #[must_use]
    pub fn new(
        id: EmployeeId,
        name: &str,
        department_id: DepartmentId,
        hire_date: Date,
        base_monthly_pay: u64,
    ) -> Self {
        Self {
            id,
            name: name.to_owned(),
            department_id,
            status: EmploymentStatus::Active,
            hire_date,
            base_monthly_pay,
        }
    }
}
