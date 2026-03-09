#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

//! ALICE-HRM: Human Resource Management
//!
//! Attendance tracking, payroll calculation, paid leave management,
//! shift scheduling, performance evaluation, and employee records.

use std::collections::HashMap;

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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
// Performance Evaluation
// ---------------------------------------------------------------------------

/// Rating scale 1-5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rating {
    NeedsImprovement = 1,
    BelowExpectations = 2,
    MeetsExpectations = 3,
    ExceedsExpectations = 4,
    Outstanding = 5,
}

impl Rating {
    #[must_use]
    pub const fn score(self) -> u32 {
        self as u32
    }
}

/// A single KPI entry.
#[derive(Debug, Clone)]
pub struct Kpi {
    pub name: String,
    pub target: f64,
    pub actual: f64,
    pub weight: f64,
}

impl Kpi {
    #[must_use]
    pub fn new(name: &str, target: f64, actual: f64, weight: f64) -> Self {
        Self {
            name: name.to_owned(),
            target,
            actual,
            weight,
        }
    }

    /// Achievement ratio (actual / target), capped at 2.0.
    #[must_use]
    pub fn achievement_ratio(&self) -> f64 {
        if self.target <= 0.0 {
            return 0.0;
        }
        (self.actual / self.target).min(2.0)
    }

    /// Weighted score.
    #[must_use]
    pub fn weighted_score(&self) -> f64 {
        self.achievement_ratio() * self.weight
    }
}

/// Performance evaluation for a period.
#[derive(Debug, Clone)]
pub struct Evaluation {
    pub employee_id: EmployeeId,
    pub period: String,
    pub kpis: Vec<Kpi>,
    pub overall_rating: Option<Rating>,
    pub comments: String,
}

impl Evaluation {
    #[must_use]
    pub fn new(employee_id: EmployeeId, period: &str) -> Self {
        Self {
            employee_id,
            period: period.to_owned(),
            kpis: Vec::new(),
            overall_rating: None,
            comments: String::new(),
        }
    }

    /// Add a KPI entry.
    pub fn add_kpi(&mut self, kpi: Kpi) {
        self.kpis.push(kpi);
    }

    /// Composite KPI score (sum of weighted scores / sum of weights).
    #[must_use]
    pub fn composite_score(&self) -> f64 {
        let total_weight: f64 = self.kpis.iter().map(|k| k.weight).sum();
        if total_weight <= 0.0 {
            return 0.0;
        }
        let total_score: f64 = self.kpis.iter().map(Kpi::weighted_score).sum();
        total_score / total_weight
    }

    /// Derive a rating from composite score.
    #[must_use]
    pub fn derived_rating(&self) -> Rating {
        let score = self.composite_score();
        if score >= 1.5 {
            Rating::Outstanding
        } else if score >= 1.2 {
            Rating::ExceedsExpectations
        } else if score >= 0.8 {
            Rating::MeetsExpectations
        } else if score >= 0.5 {
            Rating::BelowExpectations
        } else {
            Rating::NeedsImprovement
        }
    }
}

// ---------------------------------------------------------------------------
// HRM System (aggregate)
// ---------------------------------------------------------------------------

/// Central HRM system holding all records.
#[derive(Debug, Default)]
pub struct HrmSystem {
    pub employees: HashMap<EmployeeId, Employee>,
    pub attendance: Vec<AttendanceRecord>,
    pub leave_balances: HashMap<EmployeeId, LeaveBalance>,
    pub leave_requests: Vec<LeaveRequest>,
    pub schedules: HashMap<EmployeeId, WeeklySchedule>,
    pub evaluations: Vec<Evaluation>,
}

impl HrmSystem {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // -- Employee CRUD --

    /// Register a new employee. Returns false if ID already exists.
    pub fn add_employee(&mut self, emp: Employee) -> bool {
        let id = emp.id;
        if self.employees.contains_key(&id) {
            return false;
        }
        self.leave_balances.insert(id, LeaveBalance::new(id));
        self.employees.insert(id, emp);
        true
    }

    /// Get employee by id.
    #[must_use]
    pub fn get_employee(&self, id: EmployeeId) -> Option<&Employee> {
        self.employees.get(&id)
    }

    /// Update employee status.
    pub fn set_status(&mut self, id: EmployeeId, status: EmploymentStatus) -> bool {
        if let Some(emp) = self.employees.get_mut(&id) {
            emp.status = status;
            true
        } else {
            false
        }
    }

    /// Remove an employee.
    pub fn remove_employee(&mut self, id: EmployeeId) -> Option<Employee> {
        self.leave_balances.remove(&id);
        self.schedules.remove(&id);
        self.employees.remove(&id)
    }

    /// List employees in a department.
    #[must_use]
    pub fn employees_in_department(&self, dept: DepartmentId) -> Vec<&Employee> {
        self.employees
            .values()
            .filter(|e| e.department_id == dept)
            .collect()
    }

    /// Count active employees.
    #[must_use]
    pub fn active_employee_count(&self) -> usize {
        self.employees
            .values()
            .filter(|e| e.status == EmploymentStatus::Active)
            .count()
    }

    // -- Attendance --

    /// Clock in an employee.
    pub fn clock_in(&mut self, employee_id: EmployeeId, date: Date, time: Time) {
        let mut rec = AttendanceRecord::new(employee_id, date);
        rec.clock_in = Some(time);
        self.attendance.push(rec);
    }

    /// Clock out an employee (updates last record for that employee+date).
    pub fn clock_out(&mut self, employee_id: EmployeeId, date: Date, time: Time) -> bool {
        for rec in self.attendance.iter_mut().rev() {
            if rec.employee_id == employee_id && rec.date == date && rec.clock_out.is_none() {
                rec.clock_out = Some(time);
                return true;
            }
        }
        false
    }

    /// Get attendance records for an employee on a date.
    #[must_use]
    pub fn get_attendance(&self, employee_id: EmployeeId, date: Date) -> Vec<&AttendanceRecord> {
        self.attendance
            .iter()
            .filter(|r| r.employee_id == employee_id && r.date == date)
            .collect()
    }

    /// Total worked minutes for an employee across all records.
    #[must_use]
    pub fn total_worked_minutes(&self, employee_id: EmployeeId) -> u32 {
        self.attendance
            .iter()
            .filter(|r| r.employee_id == employee_id)
            .map(AttendanceRecord::worked_minutes)
            .sum()
    }

    /// Total overtime minutes for an employee.
    #[must_use]
    pub fn total_overtime_minutes(&self, employee_id: EmployeeId, standard: u32) -> u32 {
        self.attendance
            .iter()
            .filter(|r| r.employee_id == employee_id)
            .map(|r| r.overtime_minutes(standard))
            .sum()
    }

    // -- Leave --

    /// Accrue leave days.
    pub fn accrue_leave(&mut self, employee_id: EmployeeId, days: u32) -> bool {
        self.leave_balances
            .get_mut(&employee_id)
            .is_some_and(|bal| {
                bal.accrue(days);
                true
            })
    }

    /// Submit a leave request.
    pub fn submit_leave_request(&mut self, request: LeaveRequest) -> bool {
        if let Some(bal) = self.leave_balances.get_mut(&request.employee_id) {
            if bal.use_days(request.days) {
                self.leave_requests.push(request);
                return true;
            }
        }
        false
    }

    /// Get leave balance for an employee.
    #[must_use]
    pub fn get_leave_balance(&self, employee_id: EmployeeId) -> Option<&LeaveBalance> {
        self.leave_balances.get(&employee_id)
    }

    // -- Scheduling --

    /// Set a weekly schedule for an employee.
    pub fn set_schedule(&mut self, schedule: WeeklySchedule) {
        self.schedules.insert(schedule.employee_id, schedule);
    }

    /// Get the schedule for an employee.
    #[must_use]
    pub fn get_schedule(&self, employee_id: EmployeeId) -> Option<&WeeklySchedule> {
        self.schedules.get(&employee_id)
    }

    // -- Evaluation --

    /// Add an evaluation.
    pub fn add_evaluation(&mut self, eval: Evaluation) {
        self.evaluations.push(eval);
    }

    /// Get evaluations for an employee.
    #[must_use]
    pub fn get_evaluations(&self, employee_id: EmployeeId) -> Vec<&Evaluation> {
        self.evaluations
            .iter()
            .filter(|e| e.employee_id == employee_id)
            .collect()
    }
}

// ===========================================================================
// Tests
// ===========================================================================
#[cfg(test)]
mod tests {
    use super::*;

    // -- Helpers --

    fn sample_employee(id: EmployeeId) -> Employee {
        Employee::new(id, "Test Employee", 1, Date::new(2025, 4, 1), 300_000)
    }

    fn default_brackets() -> Vec<TaxBracket> {
        vec![
            TaxBracket {
                upper_bound: 195_0000,
                rate_percent: 5,
            },
            TaxBracket {
                upper_bound: 330_0000,
                rate_percent: 10,
            },
            TaxBracket {
                upper_bound: 695_0000,
                rate_percent: 20,
            },
            TaxBracket {
                upper_bound: 900_0000,
                rate_percent: 23,
            },
            TaxBracket {
                upper_bound: 1800_0000,
                rate_percent: 33,
            },
        ]
    }

    fn default_deductions() -> Deductions {
        Deductions {
            health_insurance: 15_000,
            pension: 27_000,
            employment_insurance: 900,
            other: 0,
        }
    }

    // -----------------------------------------------------------------------
    // Date tests
    // -----------------------------------------------------------------------

    #[test]
    fn date_new() {
        let d = Date::new(2026, 3, 9);
        assert_eq!(d.year, 2026);
        assert_eq!(d.month, 3);
        assert_eq!(d.day, 9);
    }

    #[test]
    fn date_equality() {
        assert_eq!(Date::new(2026, 1, 1), Date::new(2026, 1, 1));
        assert_ne!(Date::new(2026, 1, 1), Date::new(2026, 1, 2));
    }

    #[test]
    fn date_ordering() {
        assert!(Date::new(2025, 12, 31) < Date::new(2026, 1, 1));
    }

    // -----------------------------------------------------------------------
    // Time tests
    // -----------------------------------------------------------------------

    #[test]
    fn time_total_minutes() {
        assert_eq!(Time::new(9, 0).total_minutes(), 540);
        assert_eq!(Time::new(0, 0).total_minutes(), 0);
        assert_eq!(Time::new(23, 59).total_minutes(), 1439);
    }

    #[test]
    fn time_diff_minutes() {
        assert_eq!(Time::new(18, 0).diff_minutes(Time::new(9, 0)), 540);
        assert_eq!(Time::new(9, 0).diff_minutes(Time::new(18, 0)), 0);
    }

    #[test]
    fn time_diff_same() {
        assert_eq!(Time::new(12, 0).diff_minutes(Time::new(12, 0)), 0);
    }

    #[test]
    fn time_ordering() {
        assert!(Time::new(8, 30) < Time::new(9, 0));
    }

    // -----------------------------------------------------------------------
    // Employee tests
    // -----------------------------------------------------------------------

    #[test]
    fn employee_new() {
        let e = sample_employee(1);
        assert_eq!(e.id, 1);
        assert_eq!(e.name, "Test Employee");
        assert_eq!(e.status, EmploymentStatus::Active);
    }

    #[test]
    fn employee_different_ids() {
        let a = sample_employee(1);
        let b = sample_employee(2);
        assert_ne!(a.id, b.id);
    }

    // -----------------------------------------------------------------------
    // Attendance tests
    // -----------------------------------------------------------------------

    #[test]
    fn attendance_no_clock() {
        let rec = AttendanceRecord::new(1, Date::new(2026, 3, 1));
        assert_eq!(rec.worked_minutes(), 0);
    }

    #[test]
    fn attendance_worked_minutes() {
        let mut rec = AttendanceRecord::new(1, Date::new(2026, 3, 1));
        rec.clock_in = Some(Time::new(9, 0));
        rec.clock_out = Some(Time::new(18, 0));
        assert_eq!(rec.worked_minutes(), 540);
    }

    #[test]
    fn attendance_overtime() {
        let mut rec = AttendanceRecord::new(1, Date::new(2026, 3, 1));
        rec.clock_in = Some(Time::new(9, 0));
        rec.clock_out = Some(Time::new(20, 0));
        assert_eq!(rec.overtime_minutes(480), 180);
    }

    #[test]
    fn attendance_no_overtime() {
        let mut rec = AttendanceRecord::new(1, Date::new(2026, 3, 1));
        rec.clock_in = Some(Time::new(9, 0));
        rec.clock_out = Some(Time::new(17, 0));
        assert_eq!(rec.overtime_minutes(480), 0);
    }

    #[test]
    fn attendance_only_clock_in() {
        let mut rec = AttendanceRecord::new(1, Date::new(2026, 3, 1));
        rec.clock_in = Some(Time::new(9, 0));
        assert_eq!(rec.worked_minutes(), 0);
    }

    #[test]
    fn attendance_only_clock_out() {
        let mut rec = AttendanceRecord::new(1, Date::new(2026, 3, 1));
        rec.clock_out = Some(Time::new(18, 0));
        assert_eq!(rec.worked_minutes(), 0);
    }

    // -----------------------------------------------------------------------
    // Tax / Payroll tests
    // -----------------------------------------------------------------------

    #[test]
    fn tax_zero_income() {
        assert_eq!(calculate_tax(0, &default_brackets()), 0);
    }

    #[test]
    fn tax_single_bracket() {
        let brackets = vec![TaxBracket {
            upper_bound: 1_000_000,
            rate_percent: 10,
        }];
        assert_eq!(calculate_tax(500_000, &brackets), 50_000);
    }

    #[test]
    fn tax_progressive() {
        let brackets = vec![
            TaxBracket {
                upper_bound: 100,
                rate_percent: 10,
            },
            TaxBracket {
                upper_bound: 200,
                rate_percent: 20,
            },
        ];
        // First 100 at 10% = 10, next 50 at 20% = 10 => 20
        assert_eq!(calculate_tax(150, &brackets), 20);
    }

    #[test]
    fn tax_exact_boundary() {
        let brackets = vec![
            TaxBracket {
                upper_bound: 100,
                rate_percent: 10,
            },
            TaxBracket {
                upper_bound: 200,
                rate_percent: 20,
            },
        ];
        assert_eq!(calculate_tax(100, &brackets), 10);
    }

    #[test]
    fn tax_exceeds_all_brackets() {
        let brackets = vec![TaxBracket {
            upper_bound: 100,
            rate_percent: 10,
        }];
        // Only the first 100 is taxed
        assert_eq!(calculate_tax(500, &brackets), 10);
    }

    #[test]
    fn deductions_total() {
        let d = default_deductions();
        assert_eq!(d.total(), 42_900);
    }

    #[test]
    fn deductions_zero() {
        let d = Deductions {
            health_insurance: 0,
            pension: 0,
            employment_insurance: 0,
            other: 0,
        };
        assert_eq!(d.total(), 0);
    }

    #[test]
    fn payslip_basic() {
        let emp = sample_employee(1);
        let slip = compute_payslip(
            &emp,
            2026,
            3,
            0,
            125,
            default_deductions(),
            &default_brackets(),
        );
        assert_eq!(slip.base_pay, 300_000);
        assert_eq!(slip.overtime_pay, 0);
        assert!(slip.net_pay > 0);
    }

    #[test]
    fn payslip_with_overtime() {
        let emp = sample_employee(1);
        let slip = compute_payslip(
            &emp,
            2026,
            3,
            120,
            125,
            default_deductions(),
            &default_brackets(),
        );
        assert!(slip.overtime_pay > 0);
        assert!(slip.net_pay > 0);
    }

    #[test]
    fn payslip_high_deduction() {
        let emp = Employee::new(1, "Low Pay", 1, Date::new(2025, 4, 1), 50_000);
        let d = Deductions {
            health_insurance: 20_000,
            pension: 20_000,
            employment_insurance: 5_000,
            other: 5_000,
        };
        let slip = compute_payslip(&emp, 2026, 3, 0, 125, d, &default_brackets());
        assert_eq!(slip.net_pay, 0);
    }

    // -----------------------------------------------------------------------
    // Leave tests
    // -----------------------------------------------------------------------

    #[test]
    fn leave_balance_new() {
        let lb = LeaveBalance::new(1);
        assert_eq!(lb.remaining(), 0);
    }

    #[test]
    fn leave_accrue() {
        let mut lb = LeaveBalance::new(1);
        lb.accrue(10);
        assert_eq!(lb.remaining(), 10);
    }

    #[test]
    fn leave_use_ok() {
        let mut lb = LeaveBalance::new(1);
        lb.accrue(10);
        assert!(lb.use_days(5));
        assert_eq!(lb.remaining(), 5);
    }

    #[test]
    fn leave_use_insufficient() {
        let mut lb = LeaveBalance::new(1);
        lb.accrue(3);
        assert!(!lb.use_days(5));
        assert_eq!(lb.remaining(), 3);
    }

    #[test]
    fn leave_use_exact() {
        let mut lb = LeaveBalance::new(1);
        lb.accrue(5);
        assert!(lb.use_days(5));
        assert_eq!(lb.remaining(), 0);
    }

    #[test]
    fn leave_multiple_accruals() {
        let mut lb = LeaveBalance::new(1);
        lb.accrue(5);
        lb.accrue(3);
        lb.accrue(2);
        assert_eq!(lb.remaining(), 10);
    }

    #[test]
    fn leave_type_equality() {
        assert_eq!(LeaveType::Annual, LeaveType::Annual);
        assert_ne!(LeaveType::Annual, LeaveType::Sick);
    }

    // -----------------------------------------------------------------------
    // Shift / Schedule tests
    // -----------------------------------------------------------------------

    #[test]
    fn shift_duration() {
        let s = Shift::new(Time::new(9, 0), Time::new(17, 0));
        assert_eq!(s.duration_minutes(), 480);
    }

    #[test]
    fn shift_short() {
        let s = Shift::new(Time::new(12, 0), Time::new(13, 30));
        assert_eq!(s.duration_minutes(), 90);
    }

    #[test]
    fn schedule_empty() {
        let ws = WeeklySchedule::new(1);
        assert_eq!(ws.total_weekly_minutes(), 0);
        assert_eq!(ws.working_days(), 0);
    }

    #[test]
    fn schedule_assign() {
        let mut ws = WeeklySchedule::new(1);
        ws.assign(
            DayOfWeek::Monday,
            Shift::new(Time::new(9, 0), Time::new(17, 0)),
        );
        assert_eq!(ws.working_days(), 1);
        assert_eq!(ws.total_weekly_minutes(), 480);
    }

    #[test]
    fn schedule_full_week() {
        let mut ws = WeeklySchedule::new(1);
        let shift = Shift::new(Time::new(9, 0), Time::new(17, 0));
        for day in [
            DayOfWeek::Monday,
            DayOfWeek::Tuesday,
            DayOfWeek::Wednesday,
            DayOfWeek::Thursday,
            DayOfWeek::Friday,
        ] {
            ws.assign(day, shift);
        }
        assert_eq!(ws.working_days(), 5);
        assert_eq!(ws.total_weekly_minutes(), 2400);
    }

    #[test]
    fn schedule_remove() {
        let mut ws = WeeklySchedule::new(1);
        ws.assign(
            DayOfWeek::Monday,
            Shift::new(Time::new(9, 0), Time::new(17, 0)),
        );
        let removed = ws.remove(DayOfWeek::Monday);
        assert!(removed.is_some());
        assert_eq!(ws.working_days(), 0);
    }

    #[test]
    fn schedule_remove_nonexistent() {
        let mut ws = WeeklySchedule::new(1);
        assert!(ws.remove(DayOfWeek::Sunday).is_none());
    }

    #[test]
    fn schedule_overwrite() {
        let mut ws = WeeklySchedule::new(1);
        ws.assign(
            DayOfWeek::Monday,
            Shift::new(Time::new(9, 0), Time::new(17, 0)),
        );
        ws.assign(
            DayOfWeek::Monday,
            Shift::new(Time::new(10, 0), Time::new(18, 0)),
        );
        assert_eq!(ws.working_days(), 1);
        assert_eq!(ws.total_weekly_minutes(), 480);
    }

    // -----------------------------------------------------------------------
    // Rating / KPI / Evaluation tests
    // -----------------------------------------------------------------------

    #[test]
    fn rating_scores() {
        assert_eq!(Rating::NeedsImprovement.score(), 1);
        assert_eq!(Rating::Outstanding.score(), 5);
    }

    #[test]
    fn rating_ordering() {
        assert!(Rating::NeedsImprovement < Rating::Outstanding);
    }

    #[test]
    fn kpi_achievement_ratio() {
        let k = Kpi::new("Sales", 100.0, 80.0, 1.0);
        let ratio = k.achievement_ratio();
        assert!((ratio - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn kpi_achievement_cap() {
        let k = Kpi::new("Sales", 100.0, 300.0, 1.0);
        assert!((k.achievement_ratio() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn kpi_zero_target() {
        let k = Kpi::new("N/A", 0.0, 50.0, 1.0);
        assert!((k.achievement_ratio()).abs() < f64::EPSILON);
    }

    #[test]
    fn kpi_weighted_score() {
        let k = Kpi::new("Sales", 100.0, 100.0, 0.5);
        assert!((k.weighted_score() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn evaluation_empty() {
        let ev = Evaluation::new(1, "2026Q1");
        assert!((ev.composite_score()).abs() < f64::EPSILON);
    }

    #[test]
    fn evaluation_single_kpi() {
        let mut ev = Evaluation::new(1, "2026Q1");
        ev.add_kpi(Kpi::new("Sales", 100.0, 100.0, 1.0));
        assert!((ev.composite_score() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn evaluation_multiple_kpis() {
        let mut ev = Evaluation::new(1, "2026Q1");
        ev.add_kpi(Kpi::new("Sales", 100.0, 80.0, 0.6));
        ev.add_kpi(Kpi::new("Quality", 100.0, 120.0, 0.4));
        let expected = (0.8 * 0.6 + 1.2 * 0.4) / 1.0;
        assert!((ev.composite_score() - expected).abs() < 0.001);
    }

    #[test]
    fn evaluation_derived_rating_outstanding() {
        let mut ev = Evaluation::new(1, "2026Q1");
        ev.add_kpi(Kpi::new("Sales", 100.0, 200.0, 1.0));
        assert_eq!(ev.derived_rating(), Rating::Outstanding);
    }

    #[test]
    fn evaluation_derived_rating_meets() {
        let mut ev = Evaluation::new(1, "2026Q1");
        ev.add_kpi(Kpi::new("Sales", 100.0, 100.0, 1.0));
        assert_eq!(ev.derived_rating(), Rating::MeetsExpectations);
    }

    #[test]
    fn evaluation_derived_rating_below() {
        let mut ev = Evaluation::new(1, "2026Q1");
        ev.add_kpi(Kpi::new("Sales", 100.0, 60.0, 1.0));
        assert_eq!(ev.derived_rating(), Rating::BelowExpectations);
    }

    #[test]
    fn evaluation_derived_rating_needs_improvement() {
        let mut ev = Evaluation::new(1, "2026Q1");
        ev.add_kpi(Kpi::new("Sales", 100.0, 10.0, 1.0));
        assert_eq!(ev.derived_rating(), Rating::NeedsImprovement);
    }

    // -----------------------------------------------------------------------
    // HRM System tests
    // -----------------------------------------------------------------------

    #[test]
    fn hrm_add_employee() {
        let mut sys = HrmSystem::new();
        assert!(sys.add_employee(sample_employee(1)));
        assert_eq!(sys.employees.len(), 1);
    }

    #[test]
    fn hrm_add_duplicate() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        assert!(!sys.add_employee(sample_employee(1)));
    }

    #[test]
    fn hrm_get_employee() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        assert!(sys.get_employee(1).is_some());
        assert!(sys.get_employee(99).is_none());
    }

    #[test]
    fn hrm_set_status() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        assert!(sys.set_status(1, EmploymentStatus::Terminated));
        assert_eq!(
            sys.get_employee(1).unwrap().status,
            EmploymentStatus::Terminated
        );
    }

    #[test]
    fn hrm_set_status_nonexistent() {
        let mut sys = HrmSystem::new();
        assert!(!sys.set_status(99, EmploymentStatus::Active));
    }

    #[test]
    fn hrm_remove_employee() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        let removed = sys.remove_employee(1);
        assert!(removed.is_some());
        assert!(sys.get_employee(1).is_none());
        assert!(sys.leave_balances.get(&1).is_none());
    }

    #[test]
    fn hrm_remove_nonexistent() {
        let mut sys = HrmSystem::new();
        assert!(sys.remove_employee(99).is_none());
    }

    #[test]
    fn hrm_employees_in_department() {
        let mut sys = HrmSystem::new();
        sys.add_employee(Employee::new(1, "A", 10, Date::new(2025, 1, 1), 300_000));
        sys.add_employee(Employee::new(2, "B", 10, Date::new(2025, 1, 1), 300_000));
        sys.add_employee(Employee::new(3, "C", 20, Date::new(2025, 1, 1), 300_000));
        assert_eq!(sys.employees_in_department(10).len(), 2);
        assert_eq!(sys.employees_in_department(20).len(), 1);
        assert_eq!(sys.employees_in_department(30).len(), 0);
    }

    #[test]
    fn hrm_active_count() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        sys.add_employee(sample_employee(2));
        sys.set_status(2, EmploymentStatus::OnLeave);
        assert_eq!(sys.active_employee_count(), 1);
    }

    #[test]
    fn hrm_clock_in_out() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        let date = Date::new(2026, 3, 1);
        sys.clock_in(1, date, Time::new(9, 0));
        assert!(sys.clock_out(1, date, Time::new(18, 0)));
        let recs = sys.get_attendance(1, date);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].worked_minutes(), 540);
    }

    #[test]
    fn hrm_clock_out_no_in() {
        let mut sys = HrmSystem::new();
        assert!(!sys.clock_out(1, Date::new(2026, 3, 1), Time::new(18, 0)));
    }

    #[test]
    fn hrm_total_worked_minutes() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        let d1 = Date::new(2026, 3, 1);
        let d2 = Date::new(2026, 3, 2);
        sys.clock_in(1, d1, Time::new(9, 0));
        sys.clock_out(1, d1, Time::new(18, 0));
        sys.clock_in(1, d2, Time::new(9, 0));
        sys.clock_out(1, d2, Time::new(17, 0));
        assert_eq!(sys.total_worked_minutes(1), 1020); // 540 + 480
    }

    #[test]
    fn hrm_total_overtime() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        let d1 = Date::new(2026, 3, 1);
        sys.clock_in(1, d1, Time::new(9, 0));
        sys.clock_out(1, d1, Time::new(20, 0));
        assert_eq!(sys.total_overtime_minutes(1, 480), 180);
    }

    #[test]
    fn hrm_leave_accrue_and_use() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        assert!(sys.accrue_leave(1, 20));
        let req = LeaveRequest {
            employee_id: 1,
            leave_type: LeaveType::Annual,
            start_date: Date::new(2026, 4, 1),
            end_date: Date::new(2026, 4, 5),
            days: 5,
            approved: false,
        };
        assert!(sys.submit_leave_request(req));
        assert_eq!(sys.get_leave_balance(1).unwrap().remaining(), 15);
    }

    #[test]
    fn hrm_leave_insufficient() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        sys.accrue_leave(1, 2);
        let req = LeaveRequest {
            employee_id: 1,
            leave_type: LeaveType::Sick,
            start_date: Date::new(2026, 4, 1),
            end_date: Date::new(2026, 4, 5),
            days: 5,
            approved: false,
        };
        assert!(!sys.submit_leave_request(req));
    }

    #[test]
    fn hrm_leave_nonexistent_employee() {
        let mut sys = HrmSystem::new();
        assert!(!sys.accrue_leave(99, 10));
    }

    #[test]
    fn hrm_schedule_set_get() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        let mut ws = WeeklySchedule::new(1);
        ws.assign(
            DayOfWeek::Monday,
            Shift::new(Time::new(9, 0), Time::new(17, 0)),
        );
        sys.set_schedule(ws);
        let sched = sys.get_schedule(1).unwrap();
        assert_eq!(sched.working_days(), 1);
    }

    #[test]
    fn hrm_schedule_none() {
        let sys = HrmSystem::new();
        assert!(sys.get_schedule(1).is_none());
    }

    #[test]
    fn hrm_evaluation() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        let mut ev = Evaluation::new(1, "2026Q1");
        ev.add_kpi(Kpi::new("Sales", 100.0, 120.0, 1.0));
        ev.overall_rating = Some(Rating::ExceedsExpectations);
        sys.add_evaluation(ev);
        let evals = sys.get_evaluations(1);
        assert_eq!(evals.len(), 1);
        assert_eq!(evals[0].overall_rating, Some(Rating::ExceedsExpectations));
    }

    #[test]
    fn hrm_evaluation_empty() {
        let sys = HrmSystem::new();
        assert!(sys.get_evaluations(1).is_empty());
    }

    // -----------------------------------------------------------------------
    // Edge case / integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn payslip_zero_overtime_rate() {
        let emp = sample_employee(1);
        let slip = compute_payslip(
            &emp,
            2026,
            3,
            60,
            0,
            default_deductions(),
            &default_brackets(),
        );
        assert_eq!(slip.overtime_pay, 0);
    }

    #[test]
    fn payslip_employee_id_matches() {
        let emp = sample_employee(42);
        let slip = compute_payslip(
            &emp,
            2026,
            3,
            0,
            125,
            default_deductions(),
            &default_brackets(),
        );
        assert_eq!(slip.employee_id, 42);
    }

    #[test]
    fn payslip_year_month() {
        let emp = sample_employee(1);
        let slip = compute_payslip(
            &emp,
            2026,
            12,
            0,
            125,
            default_deductions(),
            &default_brackets(),
        );
        assert_eq!(slip.year, 2026);
        assert_eq!(slip.month, 12);
    }

    #[test]
    fn leave_balance_saturating() {
        let mut lb = LeaveBalance::new(1);
        lb.accrue(u32::MAX);
        lb.accrue(1);
        assert_eq!(lb.accrued_days, u32::MAX);
    }

    #[test]
    fn leave_request_approved_field() {
        let req = LeaveRequest {
            employee_id: 1,
            leave_type: LeaveType::Maternity,
            start_date: Date::new(2026, 6, 1),
            end_date: Date::new(2026, 8, 31),
            days: 90,
            approved: true,
        };
        assert!(req.approved);
        assert_eq!(req.leave_type, LeaveType::Maternity);
    }

    #[test]
    fn leave_type_all_variants() {
        let types = [
            LeaveType::Annual,
            LeaveType::Sick,
            LeaveType::Personal,
            LeaveType::Maternity,
            LeaveType::Paternity,
        ];
        assert_eq!(types.len(), 5);
    }

    #[test]
    fn employment_status_variants() {
        assert_ne!(EmploymentStatus::Active, EmploymentStatus::OnLeave);
        assert_ne!(EmploymentStatus::Active, EmploymentStatus::Terminated);
        assert_ne!(EmploymentStatus::OnLeave, EmploymentStatus::Terminated);
    }

    #[test]
    fn day_of_week_all() {
        let days = [
            DayOfWeek::Monday,
            DayOfWeek::Tuesday,
            DayOfWeek::Wednesday,
            DayOfWeek::Thursday,
            DayOfWeek::Friday,
            DayOfWeek::Saturday,
            DayOfWeek::Sunday,
        ];
        assert_eq!(days.len(), 7);
    }

    #[test]
    fn shift_equality() {
        let a = Shift::new(Time::new(9, 0), Time::new(17, 0));
        let b = Shift::new(Time::new(9, 0), Time::new(17, 0));
        assert_eq!(a, b);
    }

    #[test]
    fn hrm_multiple_employees_attendance() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        sys.add_employee(sample_employee(2));
        let date = Date::new(2026, 3, 1);
        sys.clock_in(1, date, Time::new(9, 0));
        sys.clock_out(1, date, Time::new(18, 0));
        sys.clock_in(2, date, Time::new(10, 0));
        sys.clock_out(2, date, Time::new(19, 0));
        assert_eq!(sys.total_worked_minutes(1), 540);
        assert_eq!(sys.total_worked_minutes(2), 540);
    }

    #[test]
    fn hrm_multiple_evaluations_one_employee() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        sys.add_evaluation(Evaluation::new(1, "2026Q1"));
        sys.add_evaluation(Evaluation::new(1, "2026Q2"));
        assert_eq!(sys.get_evaluations(1).len(), 2);
    }

    #[test]
    fn hrm_default_new_equivalent() {
        let a = HrmSystem::new();
        let b = HrmSystem::default();
        assert_eq!(a.employees.len(), b.employees.len());
    }

    #[test]
    fn date_clone() {
        let d = Date::new(2026, 1, 1);
        let d2 = d;
        assert_eq!(d, d2);
    }

    #[test]
    fn time_clone() {
        let t = Time::new(12, 30);
        let t2 = t;
        assert_eq!(t, t2);
    }

    #[test]
    fn kpi_negative_target() {
        let k = Kpi::new("N/A", -10.0, 5.0, 1.0);
        assert!((k.achievement_ratio()).abs() < f64::EPSILON);
    }

    #[test]
    fn evaluation_comments() {
        let mut ev = Evaluation::new(1, "2026Q1");
        ev.comments = "Good performance".to_owned();
        assert_eq!(ev.comments, "Good performance");
    }

    #[test]
    fn evaluation_period() {
        let ev = Evaluation::new(1, "FY2026-H1");
        assert_eq!(ev.period, "FY2026-H1");
    }

    #[test]
    fn hrm_clock_multiple_days() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        for day in 1..=5 {
            let d = Date::new(2026, 3, day);
            sys.clock_in(1, d, Time::new(9, 0));
            sys.clock_out(1, d, Time::new(17, 30));
        }
        // 5 days * 510 min = 2550
        assert_eq!(sys.total_worked_minutes(1), 2550);
    }

    #[test]
    fn hrm_leave_multiple_requests() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        sys.accrue_leave(1, 20);
        for i in 0u8..4 {
            let req = LeaveRequest {
                employee_id: 1,
                leave_type: LeaveType::Annual,
                start_date: Date::new(2026, 4 + i, 1),
                end_date: Date::new(2026, 4 + i, 5),
                days: 5,
                approved: false,
            };
            assert!(sys.submit_leave_request(req));
        }
        assert_eq!(sys.get_leave_balance(1).unwrap().remaining(), 0);
    }

    #[test]
    fn tax_empty_brackets() {
        assert_eq!(calculate_tax(100_000, &[]), 0);
    }

    #[test]
    fn schedule_weekend_only() {
        let mut ws = WeeklySchedule::new(1);
        ws.assign(
            DayOfWeek::Saturday,
            Shift::new(Time::new(10, 0), Time::new(15, 0)),
        );
        ws.assign(
            DayOfWeek::Sunday,
            Shift::new(Time::new(10, 0), Time::new(14, 0)),
        );
        assert_eq!(ws.working_days(), 2);
        assert_eq!(ws.total_weekly_minutes(), 540);
    }

    #[test]
    fn employee_name_unicode() {
        let e = Employee::new(1, "田中太郎", 1, Date::new(2025, 4, 1), 300_000);
        assert_eq!(e.name, "田中太郎");
    }

    #[test]
    fn hrm_remove_cleans_schedule() {
        let mut sys = HrmSystem::new();
        sys.add_employee(sample_employee(1));
        sys.set_schedule(WeeklySchedule::new(1));
        sys.remove_employee(1);
        assert!(sys.get_schedule(1).is_none());
    }

    #[test]
    fn evaluation_derived_exceeds() {
        let mut ev = Evaluation::new(1, "2026Q1");
        ev.add_kpi(Kpi::new("Sales", 100.0, 130.0, 1.0));
        assert_eq!(ev.derived_rating(), Rating::ExceedsExpectations);
    }

    #[test]
    fn attendance_record_date() {
        let rec = AttendanceRecord::new(5, Date::new(2026, 6, 15));
        assert_eq!(rec.employee_id, 5);
        assert_eq!(rec.date, Date::new(2026, 6, 15));
    }

    #[test]
    fn hrm_get_attendance_empty() {
        let sys = HrmSystem::new();
        assert!(sys.get_attendance(1, Date::new(2026, 1, 1)).is_empty());
    }

    #[test]
    fn hrm_total_overtime_no_records() {
        let sys = HrmSystem::new();
        assert_eq!(sys.total_overtime_minutes(1, 480), 0);
    }

    #[test]
    fn hrm_total_worked_no_records() {
        let sys = HrmSystem::new();
        assert_eq!(sys.total_worked_minutes(1), 0);
    }
}
