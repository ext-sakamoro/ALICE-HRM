//! performance.

use crate::employee::EmployeeId;

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
