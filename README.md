**English** | [日本語](README_JP.md)

# ALICE-HRM

Human Resource Management module for the ALICE ecosystem. Pure Rust implementation covering attendance tracking, payroll calculation, leave management, shift scheduling, and performance evaluation.

## Overview

| Item | Value |
|------|-------|
| **Crate** | `alice-hrm` |
| **Version** | 1.0.0 |
| **License** | AGPL-3.0 |
| **Edition** | 2021 |

## Features

- **Employee Records** — Core employee data with department, status, and hire date
- **Attendance Tracking** — Clock-in/out with working hours calculation (minutes-based)
- **Payroll Calculation** — Base pay, overtime multiplier, deductions, and net pay computation
- **Leave Management** — Paid leave balance tracking with request/approval workflow
- **Shift Scheduling** — Define and assign shifts with start/end times per employee per date
- **Performance Evaluation** — Scoring system with review periods and evaluator tracking

## Architecture

```
alice-hrm (lib.rs — single-file crate)
├── Date / Time                  # Date and time primitives (no external deps)
├── Employee / EmploymentStatus  # Employee records
├── AttendanceRecord             # Clock-in/out tracking
├── PayrollEntry                 # Salary computation
├── LeaveRequest / LeaveBalance  # Leave management
├── Shift / ShiftSchedule        # Shift planning
├── Evaluation                   # Performance reviews
└── HrmEngine                    # Top-level HR orchestrator
```

## Quick Start

```rust
use alice_hrm::{HrmEngine, Date, Time};

let mut hrm = HrmEngine::new();
let emp_id = hrm.add_employee("Alice", 1, Date::new(2025, 1, 15), 300_000);
hrm.clock_in(emp_id, Date::new(2025, 3, 10), Time::new(9, 0));
hrm.clock_out(emp_id, Date::new(2025, 3, 10), Time::new(18, 0));
```

## Build

```bash
cargo build
cargo test
cargo clippy -- -W clippy::all
```

## License

`AGPL-3.0 OR LicenseRef-Commercial` — dual-licensed. Pick either.

| Option | Terms | Use it when |
|--------|-------|-------------|
| **AGPL-3.0** | [LICENSE-AGPL](LICENSE-AGPL) — free, no reporting obligation | Your project is itself AGPL-compatible open source, or you are only using it internally |
| **Commercial License** | [LICENSE-COMMERCIAL.md](LICENSE-COMMERCIAL.md) — paid, removes the copyleft | Closed-source product, proprietary SaaS, edge / firmware distribution, plugin redistribution, or a platform NDA that forbids source disclosure |

AGPL is a strong copyleft: a product, firmware image, or service that links
`alice-hrm` and is distributed or served to users must be released under the AGPL
as well. That is intentional for the open ecosystem, and the Commercial
License exists for the cases where it is not something you are able to do.

Commercial licence enquiries: <contact@extoria.co.jp>
