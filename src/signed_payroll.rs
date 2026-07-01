//! `signed_payroll` — tamper-evident payroll + attendance ledger.
//!
//! Wraps payroll runs and attendance corrections in `Ed25519`-signed
//! records chained via `prev_hash → hash`. Any silent edit of a paid
//! wage or clocked hour breaks the chain.
//!
//! # Regulatory alignment
//!
//! - **`SOX` §404** — internal controls over financial reporting;
//!   payroll is a material account subject to auditor testing.
//! - **`GDPR` Art. 30** — record of processing activities for
//!   employee data (wages, absences, evaluations).
//! - **`ISO 9001` §7.5.3** — control of documented information; the
//!   chain doubles as evidence of retention integrity.
//! - **労働基準法 §108** — 賃金台帳の 3 年保存 (5 年に段階拡大); 電子
//!   保存要件を hash chain で満たす.
//!
//! Cryptographic primitives are provided by `alice-blockchain` (`Ed25519`).

#![allow(
    clippy::doc_markdown,
    clippy::missing_panics_doc,
    clippy::too_many_arguments,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]

use alice_blockchain::signature::{KeyPair, PublicKey, Signature};

// ---------------------------------------------------------------------------
// PayrollEventKind
// ---------------------------------------------------------------------------

/// The payroll or attendance event being recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PayrollEventKind {
    /// A payroll run was finalized.
    PayrollRun,
    /// A retroactive wage correction was posted.
    WageCorrection,
    /// An attendance clock-in / clock-out record was stored.
    Attendance,
    /// Paid leave was granted or consumed.
    Leave,
    /// A bonus was paid outside the regular payroll cycle.
    Bonus,
    /// A deduction (garnishment, loan repayment) was applied.
    Deduction,
}

impl PayrollEventKind {
    /// Short code used in canonical serialization.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::PayrollRun => "RUN",
            Self::WageCorrection => "CORR",
            Self::Attendance => "ATT",
            Self::Leave => "LEAVE",
            Self::Bonus => "BONUS",
            Self::Deduction => "DED",
        }
    }
}

// ---------------------------------------------------------------------------
// PayrollRecord
// ---------------------------------------------------------------------------

/// One payroll or attendance event ready to be signed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayrollRecord {
    /// Monotonic sequence number in the trail.
    pub seq: u64,
    /// Kind of event.
    pub kind: PayrollEventKind,
    /// Unix nanosecond timestamp.
    pub timestamp_ns: u64,
    /// Employee identifier.
    pub employee_id: String,
    /// Period identifier (`2026-06`, `Q2-2026`, `2026-W27`).
    pub period: String,
    /// Signed amount in minor currency units (positive for pay, negative
    /// for deductions or reversals).
    pub amount_minor: i64,
    /// ISO-4217 currency code.
    pub currency: String,
    /// Free-form detail (department, project, garnishment reason).
    pub detail: String,
    /// Hash of the previous record (0 for the genesis record).
    pub prev_hash: u64,
}

impl PayrollRecord {
    /// Canonical byte layout used for hashing and signing.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(160);
        buf.extend_from_slice(&self.seq.to_le_bytes());
        buf.extend_from_slice(self.kind.code().as_bytes());
        buf.push(0);
        buf.extend_from_slice(&self.timestamp_ns.to_le_bytes());
        buf.extend_from_slice(self.employee_id.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.period.as_bytes());
        buf.push(0);
        buf.extend_from_slice(&self.amount_minor.to_le_bytes());
        buf.extend_from_slice(self.currency.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.detail.as_bytes());
        buf.push(0);
        buf.extend_from_slice(&self.prev_hash.to_le_bytes());
        buf
    }

    /// `FNV-1a` hash of the canonical byte layout.
    #[must_use]
    pub fn hash(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in &self.canonical_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }
}

// ---------------------------------------------------------------------------
// SignedPayrollRecord
// ---------------------------------------------------------------------------

/// [`PayrollRecord`] plus the payroll officer's `Ed25519` signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedPayrollRecord {
    /// The wrapped record.
    pub record: PayrollRecord,
    /// `FNV-1a` hash of the record's canonical bytes.
    pub hash: u64,
    /// `Ed25519` signature over the canonical bytes.
    pub signature: Signature,
    /// Officer's `Ed25519` public key.
    pub officer: PublicKey,
}

impl SignedPayrollRecord {
    /// Verify signature and hash consistency.
    #[must_use]
    pub fn verify(&self) -> bool {
        if self.hash != self.record.hash() {
            return false;
        }
        self.officer
            .verify(&self.record.canonical_bytes(), &self.signature)
    }
}

// ---------------------------------------------------------------------------
// PayrollTrail
// ---------------------------------------------------------------------------

/// Append-only chain of [`SignedPayrollRecord`] records.
#[derive(Debug, Clone, Default)]
pub struct PayrollTrail {
    entries: Vec<SignedPayrollRecord>,
}

impl PayrollTrail {
    /// Construct an empty trail.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Number of entries.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the trail is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Read-only view.
    #[must_use]
    pub fn entries(&self) -> &[SignedPayrollRecord] {
        &self.entries
    }

    /// Hash of the last record (0 for empty).
    #[must_use]
    pub fn tail_hash(&self) -> u64 {
        self.entries.last().map_or(0, |e| e.hash)
    }

    /// Append a new payroll event signed with the officer's key pair.
    pub fn append(
        &mut self,
        keypair: &KeyPair,
        kind: PayrollEventKind,
        timestamp_ns: u64,
        employee_id: impl Into<String>,
        period: impl Into<String>,
        amount_minor: i64,
        currency: impl Into<String>,
        detail: impl Into<String>,
    ) -> &SignedPayrollRecord {
        let seq = self.entries.len() as u64;
        let prev_hash = self.tail_hash();
        let record = PayrollRecord {
            seq,
            kind,
            timestamp_ns,
            employee_id: employee_id.into(),
            period: period.into(),
            amount_minor,
            currency: currency.into(),
            detail: detail.into(),
            prev_hash,
        };
        let bytes = record.canonical_bytes();
        let hash = record.hash();
        let signature = keypair.sign(&bytes);
        let officer = keypair.public();
        self.entries.push(SignedPayrollRecord {
            record,
            hash,
            signature,
            officer,
        });
        self.entries.last().expect("entry was just pushed")
    }

    /// Verify signature + chain integrity end-to-end.
    #[must_use]
    pub fn find_first_tamper(&self) -> Option<usize> {
        let mut expected_prev: u64 = 0;
        for (i, e) in self.entries.iter().enumerate() {
            if e.record.seq as usize != i {
                return Some(i);
            }
            if e.record.prev_hash != expected_prev {
                return Some(i);
            }
            if !e.verify() {
                return Some(i);
            }
            expected_prev = e.hash;
        }
        None
    }

    /// Whether the trail is intact.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.find_first_tamper().is_none()
    }

    /// Sum of net paid amounts (positive amounts minus deductions) for
    /// the given employee and currency.
    #[must_use]
    pub fn employee_net(&self, employee_id: &str, currency: &str) -> i64 {
        self.entries
            .iter()
            .filter(|e| e.record.employee_id == employee_id && e.record.currency == currency)
            .map(|e| e.record.amount_minor)
            .sum()
    }

    /// Every distinct employee id seen in the trail.
    #[must_use]
    pub fn employees(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for e in &self.entries {
            if !out.contains(&e.record.employee_id) {
                out.push(e.record.employee_id.clone());
            }
        }
        out
    }

    /// Count of events of the given kind.
    #[must_use]
    pub fn count_kind(&self, kind: PayrollEventKind) -> usize {
        self.entries
            .iter()
            .filter(|e| e.record.kind == kind)
            .count()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn kp(seed: u8) -> KeyPair {
        KeyPair::from_seed([seed; 32])
    }

    #[test]
    fn kind_code_is_stable() {
        assert_eq!(PayrollEventKind::PayrollRun.code(), "RUN");
        assert_eq!(PayrollEventKind::WageCorrection.code(), "CORR");
        assert_eq!(PayrollEventKind::Attendance.code(), "ATT");
        assert_eq!(PayrollEventKind::Leave.code(), "LEAVE");
        assert_eq!(PayrollEventKind::Bonus.code(), "BONUS");
        assert_eq!(PayrollEventKind::Deduction.code(), "DED");
    }

    #[test]
    fn canonical_bytes_are_deterministic() {
        let r = PayrollRecord {
            seq: 0,
            kind: PayrollEventKind::PayrollRun,
            timestamp_ns: 1,
            employee_id: String::from("E-001"),
            period: String::from("2026-06"),
            amount_minor: 300_000,
            currency: String::from("JPY"),
            detail: String::new(),
            prev_hash: 0,
        };
        assert_eq!(r.canonical_bytes(), r.canonical_bytes());
    }

    #[test]
    fn hash_differs_when_amount_changes() {
        let mut r = PayrollRecord {
            seq: 0,
            kind: PayrollEventKind::PayrollRun,
            timestamp_ns: 1,
            employee_id: String::from("E-001"),
            period: String::from("2026-06"),
            amount_minor: 300_000,
            currency: String::from("JPY"),
            detail: String::new(),
            prev_hash: 0,
        };
        let h1 = r.hash();
        r.amount_minor = 999_999;
        assert_ne!(h1, r.hash());
    }

    #[test]
    fn empty_trail_tail_hash_is_zero() {
        let trail = PayrollTrail::new();
        assert_eq!(trail.tail_hash(), 0);
        assert!(trail.is_empty());
    }

    #[test]
    fn signed_record_verifies_on_append() {
        let mut trail = PayrollTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            PayrollEventKind::PayrollRun,
            1,
            "E-001",
            "2026-06",
            300_000,
            "JPY",
            "",
        );
        assert!(trail.entries()[0].verify());
    }

    #[test]
    fn chained_prev_hash_matches_predecessor() {
        let mut trail = PayrollTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            PayrollEventKind::PayrollRun,
            1,
            "E-001",
            "2026-06",
            300_000,
            "JPY",
            "",
        );
        trail.append(
            &k,
            PayrollEventKind::Bonus,
            2,
            "E-001",
            "2026-06",
            50_000,
            "JPY",
            "H1 bonus",
        );
        let first = trail.entries()[0].hash;
        assert_eq!(trail.entries()[1].record.prev_hash, first);
    }

    #[test]
    fn intact_trail_is_valid() {
        let mut trail = PayrollTrail::new();
        let k = kp(1);
        for i in 0..5 {
            trail.append(
                &k,
                PayrollEventKind::PayrollRun,
                i,
                "E-001",
                "2026-06",
                300_000,
                "JPY",
                "",
            );
        }
        assert!(trail.is_valid());
    }

    #[test]
    fn tampered_amount_is_detected() {
        let mut trail = PayrollTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            PayrollEventKind::PayrollRun,
            1,
            "E-001",
            "2026-06",
            300_000,
            "JPY",
            "",
        );
        // Attacker inflates the paid wage.
        trail.entries[0].record.amount_minor = 999_999_999;
        assert!(!trail.entries[0].verify());
        assert_eq!(trail.find_first_tamper(), Some(0));
    }

    #[test]
    fn tampered_employee_id_is_detected() {
        let mut trail = PayrollTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            PayrollEventKind::PayrollRun,
            1,
            "E-001",
            "2026-06",
            300_000,
            "JPY",
            "",
        );
        trail.entries[0].record.employee_id = String::from("E-attacker");
        assert!(!trail.entries[0].verify());
    }

    #[test]
    fn foreign_officer_signature_is_rejected() {
        let mut trail = PayrollTrail::new();
        let officer = kp(1);
        let attacker = kp(2);
        trail.append(
            &officer,
            PayrollEventKind::PayrollRun,
            1,
            "E-001",
            "2026-06",
            300_000,
            "JPY",
            "",
        );
        let bytes = trail.entries[0].record.canonical_bytes();
        trail.entries[0].signature = attacker.sign(&bytes);
        assert!(!trail.entries[0].verify());
    }

    #[test]
    fn employee_net_sums_positive_and_negative() {
        let mut trail = PayrollTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            PayrollEventKind::PayrollRun,
            1,
            "E-001",
            "2026-06",
            300_000,
            "JPY",
            "",
        );
        trail.append(
            &k,
            PayrollEventKind::Deduction,
            2,
            "E-001",
            "2026-06",
            -20_000,
            "JPY",
            "loan repayment",
        );
        trail.append(
            &k,
            PayrollEventKind::PayrollRun,
            3,
            "E-002",
            "2026-06",
            250_000,
            "JPY",
            "",
        );
        assert_eq!(trail.employee_net("E-001", "JPY"), 280_000);
        assert_eq!(trail.employee_net("E-002", "JPY"), 250_000);
    }

    #[test]
    fn employees_lists_distinct() {
        let mut trail = PayrollTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            PayrollEventKind::PayrollRun,
            1,
            "E-001",
            "2026-06",
            300_000,
            "JPY",
            "",
        );
        trail.append(
            &k,
            PayrollEventKind::PayrollRun,
            2,
            "E-002",
            "2026-06",
            250_000,
            "JPY",
            "",
        );
        trail.append(
            &k,
            PayrollEventKind::Bonus,
            3,
            "E-001",
            "2026-06",
            50_000,
            "JPY",
            "",
        );
        let emps = trail.employees();
        assert_eq!(emps.len(), 2);
        assert!(emps.contains(&String::from("E-001")));
        assert!(emps.contains(&String::from("E-002")));
    }

    #[test]
    fn count_kind_filters() {
        let mut trail = PayrollTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            PayrollEventKind::PayrollRun,
            1,
            "E-001",
            "2026-06",
            300_000,
            "JPY",
            "",
        );
        trail.append(
            &k,
            PayrollEventKind::Bonus,
            2,
            "E-001",
            "2026-06",
            50_000,
            "JPY",
            "",
        );
        trail.append(
            &k,
            PayrollEventKind::PayrollRun,
            3,
            "E-002",
            "2026-06",
            250_000,
            "JPY",
            "",
        );
        assert_eq!(trail.count_kind(PayrollEventKind::PayrollRun), 2);
        assert_eq!(trail.count_kind(PayrollEventKind::Bonus), 1);
        assert_eq!(trail.count_kind(PayrollEventKind::Deduction), 0);
    }

    #[test]
    fn different_kinds_produce_different_hashes() {
        let mk = |kind: PayrollEventKind| PayrollRecord {
            seq: 0,
            kind,
            timestamp_ns: 1,
            employee_id: String::new(),
            period: String::new(),
            amount_minor: 0,
            currency: String::new(),
            detail: String::new(),
            prev_hash: 0,
        };
        assert_ne!(
            mk(PayrollEventKind::PayrollRun).hash(),
            mk(PayrollEventKind::Deduction).hash()
        );
    }
}
