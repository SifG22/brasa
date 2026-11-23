use chrono::{Duration, TimeZone, Utc};

use super::*;

fn reading(length: Duration, used_percent: f32, resets_at: DateTime<Utc>) -> LimitReading {
    LimitReading {
        length,
        used_percent,
        resets_at,
    }
}

#[test]
fn codex_limits_follow_the_current_window() {
    let reset = Utc.with_ymd_and_hms(2026, 11, 28, 12, 0, 0).unwrap();
    let week = Duration::days(7);
    let sample = |day: u32, weekly: f32| LimitSample {
        time: Utc.with_ymd_and_hms(2026, 11, day, 12, 0, 0).unwrap(),
        plan: Some("plus".to_string()),
        windows: vec![
            reading(
                Duration::hours(5),
                10.,
                Utc.with_ymd_and_hms(2026, 11, day, 15, 0, 0).unwrap(),
            ),
            reading(week, weekly, reset),
        ],
    };
    let previous_week = LimitSample {
        time: Utc.with_ymd_and_hms(2026, 11, 20, 12, 0, 0).unwrap(),
        plan: Some("plus".to_string()),
        windows: vec![reading(week, 90., reset - week)],
    };
    let samples = vec![previous_week, sample(22, 2.), sample(23, 6.)];
    let now = Utc.with_ymd_and_hms(2026, 11, 23, 13, 0, 0).unwrap();

    let limits = codex_limits(&samples, now).unwrap();

    assert_eq!(limits.plan.as_deref(), Some("Plus"));
    let weekly = limits.weekly.unwrap();
    assert_eq!(weekly.used_percent, 6.);
    assert_eq!(weekly.history.len(), 2);
    assert_eq!(limits.session.unwrap().used_percent, 10.);
}

#[test]
fn a_window_that_already_reset_starts_from_zero() {
    let reset = Utc.with_ymd_and_hms(2026, 11, 20, 12, 0, 0).unwrap();
    let samples = vec![LimitSample {
        time: reset - Duration::days(1),
        plan: None,
        windows: vec![reading(Duration::days(7), 40., reset)],
    }];
    let now = reset + Duration::days(2);

    let weekly = codex_limits(&samples, now).unwrap().weekly.unwrap();

    assert_eq!(weekly.used_percent, 0.);
    assert!(weekly.expired);
}

#[test]
fn projects_usage_at_the_current_pace() {
    let resets_at = Utc.with_ymd_and_hms(2026, 11, 28, 0, 0, 0).unwrap();
    let window = LimitWindow {
        used_percent: 20.,
        resets_at,
        length: Duration::days(7),
        history: Vec::new(),
        expired: false,
    };
    // Two of seven days gone with 20% used: 70% by the reset.
    let now = resets_at - Duration::days(5);
    assert!((window.projected_percent(now) - 70.).abs() < 0.5);
    assert_eq!(window.runs_out_at(now), None);

    let busy = LimitWindow {
        used_percent: 50.,
        ..window
    };
    // 50% in two days runs out after two more.
    let runs_out = busy.runs_out_at(now).unwrap();
    assert!((runs_out - (now + Duration::days(2))).num_minutes().abs() < 5);
    assert_eq!(busy.days(now).len(), 7);
}
