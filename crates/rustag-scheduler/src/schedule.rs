//! Schedule expressions: interval (`@every`) and 5-field cron.
//!
//! Two forms are supported, covering both "simulate steady background activity"
//! (intervals) and "fire at specific wall-clock times" (cron):
//!
//! ```text
//! @every 30s          every 30 seconds
//! @every 1h30m        every 90 minutes
//! @hourly             top of every hour          (alias for `0 * * * *`)
//! @daily / @midnight  every day at 00:00         (alias for `0 0 * * *`)
//! @weekly             every Sunday at 00:00      (alias for `0 0 * * 0`)
//! */5 * * * *         every 5 minutes
//! 0 9 * * 1-5         09:00 on weekdays
//! 0 0 1 * *           midnight on the 1st of each month
//! ```
//!
//! The cron evaluator implements the standard fields
//! `minute hour day-of-month month day-of-week`, with `*`, ranges (`a-b`),
//! lists (`a,b,c`), and steps (`*/n`, `a-b/n`). Following Vixie cron, when both
//! day-of-month and day-of-week are restricted, a day matches if **either**
//! field matches. No external dependency — this keeps RustAG dependency-light.

use std::collections::BTreeSet;
use std::time::Duration;

use chrono::{DateTime, Datelike, Duration as ChronoDuration, Timelike, Utc};

use crate::error::{Result, SchedulerError};

/// A parsed schedule that can compute its next fire time after a given instant.
#[derive(Debug, Clone)]
pub enum Schedule {
    /// Fire on a fixed interval (relative to the previous fire / now).
    Every(Duration),
    /// Fire at wall-clock times matching a cron expression.
    Cron(CronSchedule),
}

impl Schedule {
    /// Parse a schedule expression.
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.is_empty() {
            return Err(SchedulerError::Schedule("empty schedule".into()));
        }
        if let Some(rest) = s.strip_prefix("@every") {
            let dur = parse_duration(rest.trim())?;
            if dur.is_zero() {
                return Err(SchedulerError::Schedule(
                    "@every interval must be greater than zero".into(),
                ));
            }
            return Ok(Schedule::Every(dur));
        }
        match s {
            "@minutely" => return Ok(Schedule::Cron(CronSchedule::parse("* * * * *")?)),
            "@hourly" => return Ok(Schedule::Cron(CronSchedule::parse("0 * * * *")?)),
            "@daily" | "@midnight" => return Ok(Schedule::Cron(CronSchedule::parse("0 0 * * *")?)),
            "@weekly" => return Ok(Schedule::Cron(CronSchedule::parse("0 0 * * 0")?)),
            "@monthly" => return Ok(Schedule::Cron(CronSchedule::parse("0 0 1 * *")?)),
            _ => {}
        }
        Ok(Schedule::Cron(CronSchedule::parse(s)?))
    }

    /// The next fire time strictly after `after`, or `None` if none exists
    /// within a year (a malformed cron that can never match).
    pub fn next_after(&self, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Schedule::Every(d) => after.checked_add_signed(ChronoDuration::from_std(*d).ok()?),
            Schedule::Cron(c) => c.next_after(after),
        }
    }
}

/// Parse a Go-style duration: a sequence of `<int><unit>` with units
/// `s`/`m`/`h`/`d` (e.g. `30s`, `5m`, `1h30m`, `2d`).
fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return Err(SchedulerError::Schedule("empty duration".into()));
    }
    let mut total: u64 = 0;
    let mut num = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            num.push(ch);
            continue;
        }
        if num.is_empty() {
            return Err(SchedulerError::Schedule(format!(
                "duration unit `{ch}` has no number"
            )));
        }
        let n: u64 = num
            .parse()
            .map_err(|_| SchedulerError::Schedule(format!("bad number in duration: {num}")))?;
        num.clear();
        let mult = match ch {
            's' => 1,
            'm' => 60,
            'h' => 3_600,
            'd' => 86_400,
            other => {
                return Err(SchedulerError::Schedule(format!(
                    "unknown duration unit `{other}` (use s/m/h/d)"
                )))
            }
        };
        total = total
            .checked_add(n.checked_mul(mult).ok_or_else(overflow)?)
            .ok_or_else(overflow)?;
    }
    if !num.is_empty() {
        return Err(SchedulerError::Schedule(format!(
            "duration `{s}` has a trailing number without a unit"
        )));
    }
    Ok(Duration::from_secs(total))
}

fn overflow() -> SchedulerError {
    SchedulerError::Schedule("duration overflow".into())
}

/// A compiled cron expression over `minute hour dom month dow`.
#[derive(Debug, Clone)]
pub struct CronSchedule {
    minute: FieldSet,
    hour: FieldSet,
    dom: FieldSet,
    month: FieldSet,
    dow: FieldSet,
    dom_restricted: bool,
    dow_restricted: bool,
}

impl CronSchedule {
    /// Parse a 5-field cron string.
    pub fn parse(spec: &str) -> Result<Self> {
        let fields: Vec<&str> = spec.split_whitespace().collect();
        if fields.len() != 5 {
            return Err(SchedulerError::Schedule(format!(
                "expected 5 cron fields, got {} in `{spec}`",
                fields.len()
            )));
        }
        let (minute, _) = FieldSet::parse(fields[0], 0, 59)?;
        let (hour, _) = FieldSet::parse(fields[1], 0, 23)?;
        let (dom, dom_restricted) = FieldSet::parse(fields[2], 1, 31)?;
        let (month, _) = FieldSet::parse(fields[3], 1, 12)?;
        let (mut dow, dow_restricted) = FieldSet::parse(fields[4], 0, 7)?;
        // Normalize Sunday: cron allows both 0 and 7.
        if dow.allowed.remove(&7) {
            dow.allowed.insert(0);
        }
        Ok(CronSchedule {
            minute,
            hour,
            dom,
            month,
            dow,
            dom_restricted,
            dow_restricted,
        })
    }

    fn matches(&self, dt: &DateTime<Utc>) -> bool {
        let dom_ok = self.dom.matches(dt.day());
        let dow_ok = self.dow.matches(dt.weekday().num_days_from_sunday());
        // Vixie-cron quirk: when both day fields are restricted, OR them.
        let day_ok = if self.dom_restricted && self.dow_restricted {
            dom_ok || dow_ok
        } else {
            dom_ok && dow_ok
        };
        self.minute.matches(dt.minute())
            && self.hour.matches(dt.hour())
            && day_ok
            && self.month.matches(dt.month())
    }

    fn next_after(&self, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        // Start at the next whole minute strictly after `after`.
        let mut dt = (after + ChronoDuration::minutes(1))
            .with_second(0)?
            .with_nanosecond(0)?;
        // Scan minute-by-minute up to ~4 years + a margin. The bound must exceed
        // the longest gap between matches of any valid cron — notably the leap-day
        // expression `0 0 29 2 *`, which fires only once every ~4 years.
        for _ in 0..(4 * 366 * 24 * 60 + 2 * 24 * 60) {
            if self.matches(&dt) {
                return Some(dt);
            }
            dt += ChronoDuration::minutes(1);
        }
        None
    }
}

/// The set of allowed values for a single cron field.
#[derive(Debug, Clone)]
struct FieldSet {
    allowed: BTreeSet<u32>,
}

impl FieldSet {
    /// Parse one field. Returns `(set, restricted)` where `restricted` is false
    /// only for the wildcard `*`.
    fn parse(spec: &str, min: u32, max: u32) -> Result<(FieldSet, bool)> {
        let spec = spec.trim();
        let restricted = spec != "*";
        let mut allowed = BTreeSet::new();
        for part in spec.split(',') {
            let part = part.trim();
            if part.is_empty() {
                return Err(SchedulerError::Schedule("empty cron list element".into()));
            }
            let (base, step) = match part.split_once('/') {
                Some((b, s)) => {
                    let step: u32 = s
                        .parse()
                        .map_err(|_| SchedulerError::Schedule(format!("bad step `{s}`")))?;
                    if step == 0 {
                        return Err(SchedulerError::Schedule("cron step cannot be 0".into()));
                    }
                    (b, step)
                }
                None => (part, 1),
            };
            let (lo, hi) = if base == "*" {
                (min, max)
            } else if let Some((a, b)) = base.split_once('-') {
                (parse_num(a)?, parse_num(b)?)
            } else {
                let v = parse_num(base)?;
                (v, v)
            };
            if lo < min || hi > max || lo > hi {
                return Err(SchedulerError::Schedule(format!(
                    "cron field value `{part}` out of range {min}-{max}"
                )));
            }
            let mut v = lo;
            while v <= hi {
                allowed.insert(v);
                v += step;
            }
        }
        if allowed.is_empty() {
            return Err(SchedulerError::Schedule(
                "cron field matched no values".into(),
            ));
        }
        Ok((FieldSet { allowed }, restricted))
    }

    fn matches(&self, v: u32) -> bool {
        self.allowed.contains(&v)
    }
}

fn parse_num(s: &str) -> Result<u32> {
    s.trim()
        .parse()
        .map_err(|_| SchedulerError::Schedule(format!("invalid cron number `{s}`")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn utc(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
    }

    #[test]
    fn parses_interval_forms() {
        assert!(
            matches!(Schedule::parse("@every 30s").unwrap(), Schedule::Every(d) if d == Duration::from_secs(30))
        );
        assert!(
            matches!(Schedule::parse("@every 1h30m").unwrap(), Schedule::Every(d) if d == Duration::from_secs(5400))
        );
        assert!(
            matches!(Schedule::parse("@every 2d").unwrap(), Schedule::Every(d) if d == Duration::from_secs(172_800))
        );
        assert!(Schedule::parse("@every 0s").is_err());
        assert!(Schedule::parse("@every 5").is_err()); // missing unit
        assert!(Schedule::parse("@every 5x").is_err()); // bad unit
    }

    #[test]
    fn interval_next_is_offset() {
        let s = Schedule::parse("@every 30s").unwrap();
        let now = utc(2026, 1, 1, 0, 0);
        assert_eq!(s.next_after(now), Some(now + ChronoDuration::seconds(30)));
    }

    #[test]
    fn cron_every_5_minutes() {
        let s = Schedule::parse("*/5 * * * *").unwrap();
        let now = utc(2026, 1, 1, 0, 2);
        assert_eq!(s.next_after(now), Some(utc(2026, 1, 1, 0, 5)));
        let exact = utc(2026, 1, 1, 0, 5);
        // strictly-after: from :05 the next is :10.
        assert_eq!(s.next_after(exact), Some(utc(2026, 1, 1, 0, 10)));
    }

    #[test]
    fn cron_weekday_morning() {
        // 09:00 Mon-Fri. 2026-01-01 is a Thursday.
        let s = Schedule::parse("0 9 * * 1-5").unwrap();
        let fri = s.next_after(utc(2026, 1, 1, 10, 0)).unwrap();
        assert_eq!(fri, utc(2026, 1, 2, 9, 0)); // Friday 09:00
                                                // From Friday 10:00, skip the weekend to Monday 09:00.
        let mon = s.next_after(utc(2026, 1, 2, 10, 0)).unwrap();
        assert_eq!(mon, utc(2026, 1, 5, 9, 0));
    }

    #[test]
    fn cron_leap_day_fires() {
        // `0 0 29 2 *` matches only on Feb 29. From 2026 the next is 2028-02-29.
        let s = Schedule::parse("0 0 29 2 *").unwrap();
        assert_eq!(
            s.next_after(utc(2026, 1, 1, 0, 0)),
            Some(utc(2028, 2, 29, 0, 0))
        );
    }

    #[test]
    fn cron_aliases_and_validation() {
        assert!(Schedule::parse("@hourly").is_ok());
        assert!(Schedule::parse("@daily").is_ok());
        assert!(Schedule::parse("0 0 1 * *").is_ok());
        assert!(Schedule::parse("0 0 * *").is_err()); // 4 fields
        assert!(Schedule::parse("99 * * * *").is_err()); // minute out of range
        assert!(Schedule::parse("*/0 * * * *").is_err()); // zero step
    }
}
