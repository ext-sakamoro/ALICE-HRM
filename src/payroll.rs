//! payroll.

use crate::employee::{Employee, EmployeeId};

// Payroll
// ---------------------------------------------------------------------------

/// Tax bracket: income up to `upper_bound` is taxed at `rate_percent`.
#[derive(Debug, Clone, Copy)]
pub struct TaxBracket {
    pub upper_bound: u64,
    pub rate_percent: u8,
}

/// Calculate income tax given a set of progressive brackets.
///
/// Brackets must be sorted by `upper_bound` ascending.
/// The last bracket's upper bound is treated as infinity.
#[must_use]
pub fn calculate_tax(gross: u64, brackets: &[TaxBracket]) -> u64 {
    let mut remaining = gross;
    let mut tax: u64 = 0;
    let mut prev_bound: u64 = 0;

    for bracket in brackets {
        if remaining == 0 {
            break;
        }
        let span = bracket.upper_bound.saturating_sub(prev_bound);
        let taxable = remaining.min(span);
        tax += taxable * u64::from(bracket.rate_percent) / 100;
        remaining = remaining.saturating_sub(taxable);
        prev_bound = bracket.upper_bound;
    }
    tax
}

/// Standard deductions applied to a payslip.
#[derive(Debug, Clone, Copy)]
pub struct Deductions {
    pub health_insurance: u64,
    pub pension: u64,
    pub employment_insurance: u64,
    pub other: u64,
}

impl Deductions {
    #[must_use]
    pub const fn total(self) -> u64 {
        self.health_insurance + self.pension + self.employment_insurance + self.other
    }
}

/// A monthly payslip.
#[derive(Debug, Clone)]
pub struct Payslip {
    pub employee_id: EmployeeId,
    pub year: u16,
    pub month: u8,
    pub base_pay: u64,
    pub overtime_pay: u64,
    pub deductions: Deductions,
    pub tax: u64,
    pub net_pay: u64,
}

/// Compute a payslip for a single month.
///
/// `overtime_minutes` is total overtime for the month.
/// `hourly_rate` is used to calculate overtime pay (typically 1.25x).
#[must_use]
pub fn compute_payslip(
    employee: &Employee,
    year: u16,
    month: u8,
    overtime_minutes: u32,
    overtime_rate_percent: u32,
    deductions: Deductions,
    brackets: &[TaxBracket],
) -> Payslip {
    let base_pay = employee.base_monthly_pay;
    // Assume 160 working hours/month for hourly rate derivation
    let hourly_rate = base_pay / 160;
    let overtime_pay =
        u64::from(overtime_minutes) * hourly_rate * u64::from(overtime_rate_percent) / 100 / 60;
    let gross = base_pay + overtime_pay;
    let taxable = gross.saturating_sub(deductions.total());
    let tax = calculate_tax(taxable, brackets);
    let net_pay = gross.saturating_sub(deductions.total()).saturating_sub(tax);

    Payslip {
        employee_id: employee.id,
        year,
        month,
        base_pay,
        overtime_pay,
        deductions,
        tax,
        net_pay,
    }
}
