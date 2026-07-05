//! ALICE-HRM: HRM system (employee/attendance/payroll/leave/shift/performance).

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::too_many_lines,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::return_self_not_must_use
)]

pub mod attendance;
pub mod date_time;
pub mod employee;
pub mod leave;
pub mod payroll;
pub mod performance;
pub mod prelude;
pub mod shift;
pub mod signed_payroll;
pub mod system;

#[cfg(test)]
mod integration_tests;

pub use crate::attendance::*;
pub use crate::date_time::*;
pub use crate::employee::*;
pub use crate::leave::*;
pub use crate::payroll::*;
pub use crate::performance::*;
pub use crate::shift::*;
pub use crate::system::*;
