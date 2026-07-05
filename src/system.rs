//! system.

use crate::attendance::*;
use crate::date_time::*;
use crate::employee::*;
use crate::leave::*;
use crate::performance::*;
use crate::shift::*;
use std::collections::HashMap;

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
