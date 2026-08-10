//! Integration tests.

#![allow(
    clippy::wildcard_imports,
    clippy::too_many_lines,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use crate::attendance::*;
use crate::date_time::*;
use crate::employee::*;
use crate::leave::*;
use crate::payroll::*;
use crate::performance::*;
use crate::shift::*;
use crate::system::*;

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
    let expected = 0.8_f64.mul_add(0.6, 1.2 * 0.4);
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
    assert!(!sys.leave_balances.contains_key(&1));
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
