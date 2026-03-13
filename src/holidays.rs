use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use tracing::debug;

use crate::holidays_fallback;

#[derive(Debug, thiserror::Error)]
pub enum HolidayError {
    #[error("no holiday data for year {year} -- run sync or use --force")]
    MissingYear { year: i32 },
}

/// Schema for both the embedded resource and the user-side `holidays.json`.
#[derive(Debug, Serialize, Deserialize)]
pub struct HolidaySnapshot {
    pub holidays: BTreeMap<String, Vec<String>>,
}

static EMBEDDED_JSON: &str = include_str!("generated/serbian_holidays.json");

pub struct HolidayCalendar {
    embedded: HashMap<i32, HashSet<NaiveDate>>,
    file_overlay: HashMap<i32, HashSet<NaiveDate>>,
    allow_fallback: bool,
}

impl HolidayCalendar {
    /// Load the calendar from the compile-time embedded resource only.
    ///
    /// # Panics
    ///
    /// Panics if the embedded holiday snapshot is invalid JSON.
    #[must_use]
    pub fn load_embedded() -> Self {
        let embedded = parse_snapshot(EMBEDDED_JSON).expect("embedded holiday snapshot is invalid");
        Self {
            embedded,
            file_overlay: HashMap::new(),
            allow_fallback: false,
        }
    }

    /// Construct an empty calendar (for tests).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            embedded: HashMap::new(),
            file_overlay: HashMap::new(),
            allow_fallback: false,
        }
    }

    /// Merge holiday data from `holidays.json` in the given directory.
    /// File data takes per-year priority over embedded data.
    /// If the file does not exist, this is a no-op.
    pub fn merge_file(&mut self, data_dir: &Path) {
        let path = data_dir.join("holidays.json");
        let Ok(content) = std::fs::read_to_string(&path) else {
            return;
        };
        match parse_snapshot(&content) {
            Ok(data) => {
                debug!(path = %path.display(), years = data.len(), "loaded holiday overlay");
                self.file_overlay = data;
            }
            Err(e) => {
                debug!(path = %path.display(), error = %e, "ignoring invalid holidays.json");
            }
        }
    }

    /// Set the fallback policy. When `true`, missing years fall back to
    /// the algorithmic calendar instead of returning an error.
    pub fn set_fallback(&mut self, allow: bool) {
        self.allow_fallback = allow;
    }

    /// Add holiday data for a year (goes into the file overlay tier).
    pub fn add_year(&mut self, year: i32, dates: Vec<NaiveDate>) {
        self.file_overlay.insert(year, dates.into_iter().collect());
    }

    /// Save only the file overlay data to `holidays.json`.
    pub fn save_overlay(&self, data_dir: &Path) -> anyhow::Result<()> {
        if self.file_overlay.is_empty() {
            return Ok(());
        }
        let snapshot = to_snapshot(&self.file_overlay);
        let json = serde_json::to_string_pretty(&snapshot)?;
        let path = data_dir.join("holidays.json");
        std::fs::write(&path, format!("{json}\n"))?;
        debug!(path = %path.display(), "saved holiday overlay");
        Ok(())
    }

    /// Check whether a date is a Serbian public holiday.
    ///
    /// Lookup order:
    /// 1. File overlay (if it has the year)
    /// 2. Embedded data (if it has the year)
    /// 3. Fallback algorithm (if `allow_fallback` is set)
    /// 4. Error
    pub fn is_serbian_holiday(&self, date: NaiveDate) -> Result<bool, HolidayError> {
        let year = date.year();

        if let Some(dates) = self.file_overlay.get(&year) {
            return Ok(dates.contains(&date));
        }
        if let Some(dates) = self.embedded.get(&year) {
            return Ok(dates.contains(&date));
        }
        if self.allow_fallback {
            debug!(%date, "year {year} not loaded, using fallback calendar");
            return Ok(holidays_fallback::is_serbian_holiday_fallback(date));
        }
        Err(HolidayError::MissingYear { year })
    }

    /// Check whether holiday data is available for a given year.
    #[must_use]
    pub fn is_year_loaded(&self, year: i32) -> bool {
        self.file_overlay.contains_key(&year) || self.embedded.contains_key(&year)
    }

    /// Check whether a date falls on a weekend (Saturday or Sunday).
    #[must_use]
    pub fn is_weekend(date: NaiveDate) -> bool {
        matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
    }

    /// List all years that have loaded holiday data (from either tier).
    #[must_use]
    pub fn loaded_years(&self) -> HashSet<i32> {
        let mut years: HashSet<i32> = self.embedded.keys().copied().collect();
        years.extend(self.file_overlay.keys());
        years
    }
}

fn parse_snapshot(json: &str) -> anyhow::Result<HashMap<i32, HashSet<NaiveDate>>> {
    let snap: HolidaySnapshot = serde_json::from_str(json)?;
    let mut result = HashMap::new();
    for (year_str, date_strs) in &snap.holidays {
        let year: i32 = year_str.parse()?;
        let dates: HashSet<NaiveDate> = date_strs
            .iter()
            .map(|s| {
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .unwrap_or_else(|e| panic!("invalid date '{s}' in holiday data: {e}"))
            })
            .collect();
        result.insert(year, dates);
    }
    Ok(result)
}

/// Convert in-memory holiday data back to the serializable snapshot format.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn to_snapshot(data: &HashMap<i32, HashSet<NaiveDate>>) -> HolidaySnapshot {
    let mut holidays = BTreeMap::new();
    for (&year, dates) in data {
        let mut sorted: Vec<String> = dates
            .iter()
            .map(|d| d.format("%Y-%m-%d").to_string())
            .collect();
        sorted.sort();
        holidays.insert(year.to_string(), sorted);
    }
    HolidaySnapshot { holidays }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_data_is_valid() {
        let cal = HolidayCalendar::load_embedded();
        assert!(cal.is_year_loaded(2025));
        assert!(
            cal.is_serbian_holiday(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
                .unwrap()
        );
    }

    #[test]
    fn test_known_holidays() {
        let cal = HolidayCalendar::load_embedded();
        let check = |y, m, d| {
            cal.is_serbian_holiday(NaiveDate::from_ymd_opt(y, m, d).unwrap())
                .unwrap()
        };
        assert!(check(2025, 1, 1));
        assert!(check(2025, 1, 7));
        assert!(check(2025, 2, 15));
        assert!(check(2025, 5, 1));
        assert!(check(2025, 11, 11));
        assert!(check(2025, 4, 18)); // Good Friday
        assert!(check(2025, 4, 20)); // Easter Sunday
        assert!(check(2025, 4, 21)); // Easter Monday
    }

    #[test]
    fn test_non_holiday() {
        let cal = HolidayCalendar::load_embedded();
        assert!(
            !cal.is_serbian_holiday(NaiveDate::from_ymd_opt(2025, 3, 12).unwrap())
                .unwrap()
        );
    }

    #[test]
    fn test_missing_year_returns_error() {
        let cal = HolidayCalendar::empty();
        let result = cal.is_serbian_holiday(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, HolidayError::MissingYear { year: 2025 }));
    }

    #[test]
    fn test_fallback_when_allowed() {
        let mut cal = HolidayCalendar::empty();
        cal.set_fallback(true);
        assert!(
            cal.is_serbian_holiday(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
                .unwrap()
        );
        assert!(
            !cal.is_serbian_holiday(NaiveDate::from_ymd_opt(2025, 3, 12).unwrap())
                .unwrap()
        );
    }

    #[test]
    fn test_file_overlay_overrides_embedded() {
        let mut cal = HolidayCalendar::load_embedded();
        // Override 2025 with empty data -- Jan 1 should no longer be a holiday
        cal.add_year(2025, vec![]);
        assert!(
            !cal.is_serbian_holiday(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
                .unwrap()
        );
        // 2026 still uses embedded
        assert!(
            cal.is_serbian_holiday(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
                .unwrap()
        );
    }

    #[test]
    fn test_save_overlay_only_writes_file_data() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut cal = HolidayCalendar::load_embedded();
        cal.add_year(2099, vec![NaiveDate::from_ymd_opt(2099, 1, 1).unwrap()]);
        cal.save_overlay(tmp.path()).unwrap();

        let saved = std::fs::read_to_string(tmp.path().join("holidays.json")).unwrap();
        let snap: HolidaySnapshot = serde_json::from_str(&saved).unwrap();
        assert_eq!(snap.holidays.len(), 1);
        assert!(snap.holidays.contains_key("2099"));
    }

    #[test]
    fn test_merge_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let snap = HolidaySnapshot {
            holidays: BTreeMap::from([("2025".to_string(), vec!["2025-06-15".to_string()])]),
        };
        let json = serde_json::to_string_pretty(&snap).unwrap();
        std::fs::write(tmp.path().join("holidays.json"), format!("{json}\n")).unwrap();

        let mut cal = HolidayCalendar::load_embedded();
        cal.merge_file(tmp.path());

        // File says 2025-06-15 is a holiday (overrides embedded for 2025)
        assert!(
            cal.is_serbian_holiday(NaiveDate::from_ymd_opt(2025, 6, 15).unwrap())
                .unwrap()
        );
        // File overrides embedded for 2025, so Jan 1 is no longer a holiday
        assert!(
            !cal.is_serbian_holiday(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
                .unwrap()
        );
    }

    #[test]
    fn test_merge_file_nonexistent_is_noop() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut cal = HolidayCalendar::load_embedded();
        cal.merge_file(tmp.path());
        assert!(cal.is_year_loaded(2025));
    }

    #[test]
    fn test_is_weekend() {
        assert!(HolidayCalendar::is_weekend(
            NaiveDate::from_ymd_opt(2025, 3, 8).unwrap()
        ));
        assert!(HolidayCalendar::is_weekend(
            NaiveDate::from_ymd_opt(2025, 3, 9).unwrap()
        ));
        assert!(!HolidayCalendar::is_weekend(
            NaiveDate::from_ymd_opt(2025, 3, 10).unwrap()
        ));
    }

    #[test]
    fn test_loaded_years() {
        let cal = HolidayCalendar::load_embedded();
        let years = cal.loaded_years();
        assert!(years.contains(&2025));
        assert!(years.contains(&2026));
    }

    #[test]
    fn test_to_snapshot_roundtrip() {
        let mut data = HashMap::new();
        data.insert(
            2025,
            HashSet::from([
                NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 1, 7).unwrap(),
            ]),
        );
        let snap = to_snapshot(&data);
        let json = serde_json::to_string(&snap).unwrap();
        let parsed = parse_snapshot(&json).unwrap();
        assert_eq!(parsed[&2025], data[&2025]);
    }
}
