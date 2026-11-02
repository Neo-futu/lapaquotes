//! Hong Kong next working day (weekdays excluding `Holiday` region=HK).

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use cornucopia::deadpool_postgres::Pool;
use std::collections::HashSet;

use super::error::BiError;

/// First calendar day **strictly after** `from` that is Mon–Fri and not a HK public holiday.
pub async fn next_hk_workday(pool: &Pool, from: NaiveDate) -> Result<NaiveDate, BiError> {
    let holidays = load_hk_holidays(pool, from, from + Duration::days(45)).await?;
    Ok(next_workday_after(from, &holidays))
}

pub fn next_workday_after(from: NaiveDate, holidays: &HashSet<NaiveDate>) -> NaiveDate {
    let mut d = from + Duration::days(1);
    for _ in 0..60 {
        if is_hk_workday(d, holidays) {
            return d;
        }
        d += Duration::days(1);
    }
    // Fallback: never block sync if holiday seed is thin.
    from + Duration::days(1)
}

pub fn is_hk_workday(d: NaiveDate, holidays: &HashSet<NaiveDate>) -> bool {
    !matches!(d.weekday(), Weekday::Sat | Weekday::Sun) && !holidays.contains(&d)
}

async fn load_hk_holidays(
    pool: &Pool,
    from: NaiveDate,
    until: NaiveDate,
) -> Result<HashSet<NaiveDate>, BiError> {
    let client = pool
        .get()
        .await
        .map_err(|e| BiError::Pool(e.to_string()))?;
    let rows = client
        .query(
            r#"
            SELECT holiday_date
            FROM "Holiday"
            WHERE region = 'HK'
              AND holiday_date > $1
              AND holiday_date <= $2
            "#,
            &[&from, &until],
        )
        .await?;
    Ok(rows.into_iter().map(|r| r.get(0)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn skips_weekend() {
        // Friday 2026-10-23 → Monday 2026-10-26
        let fri = NaiveDate::from_ymd_opt(2026, 10, 23).unwrap();
        assert_eq!(
            next_workday_after(fri, &HashSet::new()),
            NaiveDate::from_ymd_opt(2026, 10, 26).unwrap()
        );
    }

    #[test]
    fn skips_holiday() {
        // Thu 2026-12-24 → skip Fri Christmas (seeded) → Mon 28 if Sat/Sun also
        let thu = NaiveDate::from_ymd_opt(2026, 12, 24).unwrap();
        let mut h = HashSet::new();
        h.insert(NaiveDate::from_ymd_opt(2026, 12, 25).unwrap());
        // 25 Fri holiday, 26 Sat, 27 Sun → Mon 28
        assert_eq!(
            next_workday_after(thu, &h),
            NaiveDate::from_ymd_opt(2026, 12, 28).unwrap()
        );
    }
}
