#![allow(unused)]
use chrono::{DateTime, TimeZone, Utc};
use chronographer::prelude::*;
use std::time::SystemTime;

fn to_dt(st: SystemTime) -> DateTime<Utc> {
    st.into()
}

#[test]
fn test_cron_every_second() {
    let c = cron!("* * * * * *");
    let now = Utc.with_ymd_and_hms(2026, 3, 14, 12, 0, 0).unwrap().into();
    let next = c.schedule(now).unwrap();
    assert_eq!(
        to_dt(next),
        Utc.with_ymd_and_hms(2026, 3, 14, 12, 0, 1).unwrap()
    );
}

#[test]
fn test_cron_specific_noon() {
    let c = cron!("0 0 12 * * *");
    let now = Utc.with_ymd_and_hms(2026, 3, 14, 10, 0, 0).unwrap().into();
    let next = c.schedule(now).unwrap();
    assert_eq!(
        to_dt(next),
        Utc.with_ymd_and_hms(2026, 3, 14, 12, 0, 0).unwrap()
    );

    let now2 = Utc.with_ymd_and_hms(2026, 3, 14, 13, 0, 0).unwrap().into();
    let next2 = c.schedule(now2).unwrap();
    assert_eq!(
        to_dt(next2),
        Utc.with_ymd_and_hms(2026, 3, 15, 12, 0, 0).unwrap()
    );
}

#[test]
fn test_cron_steps() {
    let c = cron!("*/15 * * * * *");
    let now = Utc.with_ymd_and_hms(2026, 3, 14, 12, 0, 0).unwrap().into();
    let next = c.schedule(now).unwrap();
    assert_eq!(
        to_dt(next),
        Utc.with_ymd_and_hms(2026, 3, 14, 12, 0, 15).unwrap()
    );

    let now2 = Utc.with_ymd_and_hms(2026, 3, 14, 12, 0, 45).unwrap().into();
    let next2 = c.schedule(now2).unwrap();
    assert_eq!(
        to_dt(next2),
        Utc.with_ymd_and_hms(2026, 3, 14, 12, 1, 0).unwrap()
    );
}

#[test]
fn test_cron_list() {
    let c = cron!("0 0 9,12,15 * * *");
    let now = Utc.with_ymd_and_hms(2026, 3, 14, 10, 0, 0).unwrap().into();
    let next = c.schedule(now).unwrap();
    assert_eq!(
        to_dt(next),
        Utc.with_ymd_and_hms(2026, 3, 14, 12, 0, 0).unwrap()
    );
}

#[test]
fn test_cron_dow_named() {
    // MON-FRI at 9am
    let c = cron!("0 0 9 * * MON-FRI");
    // Mar 14, 2026 is Saturday (7)
    // Mar 15, 2026 is Sunday (1)
    // Mar 16, 2026 is Monday (2)
    let now = Utc.with_ymd_and_hms(2026, 3, 14, 8, 0, 0).unwrap().into();
    let next = c.schedule(now).unwrap();
    // Next Monday is Mar 16
    assert_eq!(
        to_dt(next),
        Utc.with_ymd_and_hms(2026, 3, 16, 9, 0, 0).unwrap()
    );
}

#[test]
fn test_cron_last_day_of_month() {
    let c = cron!("0 0 0 L * ?");
    let now = Utc.with_ymd_and_hms(2026, 2, 28, 0, 0, 0).unwrap().into();
    // Feb 28 is the last day, next should be Mar 31
    let next = c.schedule(now).unwrap();
    assert_eq!(
        to_dt(next),
        Utc.with_ymd_and_hms(2026, 3, 31, 0, 0, 0).unwrap()
    );

    let now2 = Utc.with_ymd_and_hms(2026, 2, 27, 0, 0, 0).unwrap().into();
    let next2 = c.schedule(now2).unwrap();
    assert_eq!(
        to_dt(next2),
        Utc.with_ymd_and_hms(2026, 2, 28, 0, 0, 0).unwrap()
    );
}

#[test]
fn test_cron_weekday_nearest() {
    // Nearest weekday to 15th
    let c = cron!("0 0 0 15W * ?");
    // Mar 15, 2026 is Sunday. Nearest is Monday 16th.
    let now = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap().into();
    let next = c.schedule(now).unwrap();
    assert_eq!(
        to_dt(next),
        Utc.with_ymd_and_hms(2026, 3, 16, 0, 0, 0).unwrap()
    );
}

#[test]
fn test_cron_nth_weekday() {
    // 3rd Friday of Month
    let c = cron!("0 0 0 ? * 6#3");
    // Mar 1, 2026 is Sunday.
    // 1st Fri: Mar 6
    // 2nd Fri: Mar 13
    // 3rd Fri: Mar 20
    let now = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap().into();
    let next = c.schedule(now).unwrap();
    assert_eq!(
        to_dt(next),
        Utc.with_ymd_and_hms(2026, 3, 20, 0, 0, 0).unwrap()
    );
}
#[test]
fn test_macro_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("ui/cron_errors.rs");
}
