use chrono::{Datelike, NaiveDate};

/// Algorithmic Serbian holiday check used as last-resort fallback
/// when the `OpenHolidays` API snapshot does not cover the requested year.
#[must_use]
pub fn is_serbian_holiday_fallback(date: NaiveDate) -> bool {
    let (month, day) = (date.month(), date.day());

    #[allow(clippy::unnested_or_patterns)]
    if matches!(
        (month, day),
        (1, 1) | (1, 2)       // New Year
        | (1, 7)               // Orthodox Christmas
        | (2, 15) | (2, 16)   // Statehood Day
        | (5, 1) | (5, 2)     // Labor Day
        | (11, 11) // Armistice Day
    ) {
        return true;
    }

    let easter = orthodox_easter(date.year());
    let good_friday = easter - chrono::Duration::days(2);
    let holy_saturday = easter - chrono::Duration::days(1);
    let easter_monday = easter + chrono::Duration::days(1);

    date == good_friday || date == holy_saturday || date == easter || date == easter_monday
}

/// Compute Orthodox Easter date for a given year (Gregorian calendar result).
///
/// Uses the Julian calendar Easter algorithm, then applies the +13 day
/// Julian-to-Gregorian offset (valid for years 1900-2099).
///
/// # Panics
///
/// Cannot panic for any valid `i32` year (March 22 always exists).
#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn orthodox_easter(year: i32) -> NaiveDate {
    let a = year % 19;
    let b = year % 4;
    let c = year % 7;
    let d = (19 * a + 15) % 30;
    let e = (2 * b + 4 * c + 6 * d + 6) % 7;

    let base = NaiveDate::from_ymd_opt(year, 3, 22).unwrap();
    let julian_easter = base + chrono::Duration::days(i64::from(d + e));
    julian_easter + chrono::Duration::days(13)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orthodox_easter_known_dates() {
        assert_eq!(
            orthodox_easter(2023),
            NaiveDate::from_ymd_opt(2023, 4, 16).unwrap()
        );
        assert_eq!(
            orthodox_easter(2024),
            NaiveDate::from_ymd_opt(2024, 5, 5).unwrap()
        );
        assert_eq!(
            orthodox_easter(2025),
            NaiveDate::from_ymd_opt(2025, 4, 20).unwrap()
        );
        assert_eq!(
            orthodox_easter(2026),
            NaiveDate::from_ymd_opt(2026, 4, 12).unwrap()
        );
    }

    #[test]
    fn test_is_serbian_holiday_fallback() {
        assert!(is_serbian_holiday_fallback(
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
        ));
        assert!(is_serbian_holiday_fallback(
            NaiveDate::from_ymd_opt(2025, 1, 7).unwrap()
        ));
        assert!(is_serbian_holiday_fallback(
            NaiveDate::from_ymd_opt(2025, 4, 18).unwrap()
        ));
        assert!(!is_serbian_holiday_fallback(
            NaiveDate::from_ymd_opt(2025, 3, 12).unwrap()
        ));
    }
}
