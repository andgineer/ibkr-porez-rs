use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use chrono::{Datelike, NaiveDate};

use ibkr_porez::holidays::{HolidaySnapshot, to_snapshot};
use ibkr_porez::openholiday::OpenHolidayClient;

const API_MIN_YEAR: i32 = 2020;

fn main() {
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("generated")
        .join("serbian_holidays.json");

    let today = chrono::Local::now().date_naive();
    let current_year = today.year();
    let target_min = (current_year - 10).max(API_MIN_YEAR);
    let target_max = if today.month() == 12 && today.day() > 15 {
        current_year + 1
    } else {
        current_year
    };

    let existing = load_existing(&output);
    let existing_years: HashSet<i32> = existing.keys().copied().collect();

    let mut missing: Vec<i32> = (target_min..=target_max)
        .filter(|y| !existing_years.contains(y))
        .collect();
    missing.sort_unstable();

    if missing.is_empty() {
        println!(
            "All years {target_min}-{target_max} already present in {}",
            output.display()
        );
        return;
    }

    println!("Fetching {} missing year(s): {:?}", missing.len(), missing);

    let client = OpenHolidayClient::new();
    let from = *missing.first().unwrap();
    let to = *missing.last().unwrap();
    let fetched = client
        .fetch_years(from, to)
        .expect("failed to fetch holidays from OpenHolidays API");

    let mut merged = existing;
    for year in &missing {
        let dates = fetched.get(year).cloned().unwrap_or_default();
        let date_set = dates.into_iter().collect();
        merged.insert(*year, date_set);
    }

    let snapshot = to_snapshot(&merged);
    let json = serde_json::to_string_pretty(&snapshot).expect("failed to serialize snapshot");

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).expect("failed to create output directory");
    }
    std::fs::write(&output, format!("{json}\n")).expect("failed to write snapshot");

    let total_dates: usize = merged.values().map(HashSet::len).sum();
    println!(
        "Wrote {total_dates} holidays across {} years to {}",
        merged.len(),
        output.display()
    );
}

fn load_existing(path: &std::path::Path) -> HashMap<i32, HashSet<NaiveDate>> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };
    let Ok(snapshot) = serde_json::from_str::<HolidaySnapshot>(&content) else {
        return HashMap::new();
    };
    let mut result = HashMap::new();
    for (year_str, date_strs) in &snapshot.holidays {
        let Ok(year) = year_str.parse::<i32>() else {
            continue;
        };
        let dates: HashSet<NaiveDate> = date_strs
            .iter()
            .filter_map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .collect();
        result.insert(year, dates);
    }
    result
}
