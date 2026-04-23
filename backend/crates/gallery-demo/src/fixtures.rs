//! Shared synthetic-row generator for CAT-03 (n=500) and Phase 19 EXER-03 (n=10_000).
//!
//! Deterministic: same `n` always yields the same rows (seeded LCG). No dependency
//! on `rand` — keeps gallery-demo crate-weight minimal. Generator idiom:
//! Numerical-Recipes LCG constants (a=1664525, c=1013904223, m=2^64 via wrapping).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub status: Status,
    pub score: i32,
    pub joined_at: NaiveDate,
}

/// Generate `n` deterministic synthetic rows. Same `n` → same rows.
///
/// # Panics
///
/// Panics only if the hardcoded base date `2024-01-01` fails to construct,
/// which is impossible for a compile-time-known valid date.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn synthetic_rows(n: usize) -> Vec<Row> {
    let mut state: u64 = 0x1234_5678_9ABC_DEF0;
    let mut rng = || {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        state
    };
    let first_names = [
        "Alice", "Bob", "Carol", "Dan", "Eva", "Frank", "Gina", "Henry", "Iris",
        "Jack", "Kara", "Leo", "Maya", "Noah", "Olive", "Paul",
    ];
    let last_names = [
        "Baker", "Chen", "Davis", "Evans", "Frost", "Gomez", "Hale", "Iqbal",
    ];
    let statuses = [Status::Active, Status::Inactive, Status::Pending];
    let base_date = NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date");

    (1..=n as u64)
        .map(|id| {
            let f = first_names[(rng() as usize) % first_names.len()];
            let l = last_names[(rng() as usize) % last_names.len()];
            let st = statuses[(rng() as usize) % statuses.len()].clone();
            let sc = (rng() % 1000) as i32;
            let days = (rng() % 700) as i64;
            let joined = base_date + chrono::Duration::days(days);
            Row {
                id,
                name: format!("{f} {l}"),
                email: format!(
                    "{}.{}@example.com",
                    f.to_lowercase(),
                    l.to_lowercase()
                ),
                status: st,
                score: sc,
                joined_at: joined,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_length() {
        assert_eq!(synthetic_rows(500).len(), 500);
        assert_eq!(synthetic_rows(0).len(), 0);
        assert_eq!(synthetic_rows(1).len(), 1);
    }

    #[test]
    fn generator_id_starts_at_one_and_increments() {
        let rows = synthetic_rows(10);
        assert_eq!(rows[0].id, 1);
        assert_eq!(rows[9].id, 10);
    }

    #[test]
    fn generator_deterministic_across_calls() {
        // Same n → same Vec<Row> (structurally equal after serialize).
        let a = serde_json::to_value(synthetic_rows(500)).expect("serialize");
        let b = serde_json::to_value(synthetic_rows(500)).expect("serialize");
        assert_eq!(a, b, "synthetic_rows must be deterministic");
    }

    #[test]
    fn generator_status_serializes_as_lowercase_string() {
        let rows = synthetic_rows(50);
        let any_row = serde_json::to_value(&rows[0]).expect("serialize");
        let s = any_row["status"].as_str().expect("status is string");
        assert!(
            matches!(s, "active" | "inactive" | "pending"),
            "status must be one of active/inactive/pending, got {s}"
        );
    }

    #[test]
    fn generator_joined_at_is_iso_date() {
        let row = &synthetic_rows(1)[0];
        let v = serde_json::to_value(row).expect("serialize");
        let s = v["joined_at"].as_str().expect("joined_at is string");
        // chrono NaiveDate serializes as YYYY-MM-DD by default
        assert_eq!(s.len(), 10);
        assert_eq!(&s[4..5], "-");
        assert_eq!(&s[7..8], "-");
    }
}
