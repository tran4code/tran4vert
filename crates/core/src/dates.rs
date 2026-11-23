//! Dates as whole numbers of days, with no clock. The page passes today's date
//! as text; the core does every sum with it. A day number counts days from
//! 1970-01-01, which was a Thursday. The conversions follow Howard Hinnant's
//! civil-calendar algorithms, which hold for any year in the proleptic
//! Gregorian calendar.

/// Days from 1970-01-01 for a date written `YYYY-MM-DD`, or `None` when the
/// text is not a real date (month 13, 30 February, a missing zero).
pub fn parse(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    let shaped = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit());
    if !shaped {
        return None;
    }
    let year: i64 = text[0..4].parse().ok()?;
    let month: i64 = text[5..7].parse().ok()?;
    let day: i64 = text[8..10].parse().ok()?;
    if !(1..=12).contains(&month) || day < 1 {
        return None;
    }
    let number = from_civil(year, month, day);
    // A day past the end of the month comes back as a different date.
    if format(number) == text {
        Some(number)
    } else {
        None
    }
}

fn from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let shifted = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * shifted + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Year, month (1 to 12) and day of month for a day number.
pub fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

/// `YYYY-MM-DD` for a day number.
pub fn format(days: i64) -> String {
    let (y, m, d) = civil(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// 0 for Monday up to 6 for Sunday.
pub fn weekday(days: i64) -> u8 {
    (days + 3).rem_euclid(7) as u8
}

/// The Monday on or before a day.
pub fn monday_on_or_before(days: i64) -> i64 {
    days - i64::from(weekday(days))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_epoch_is_day_zero_and_a_thursday() {
        assert_eq!(parse("1970-01-01"), Some(0));
        assert_eq!(weekday(0), 3);
    }

    #[test]
    fn the_first_of_july_2024_is_a_monday() {
        let d = parse("2024-07-01").unwrap();
        assert_eq!(weekday(d), 0);
        // 2024-07-06 is the Saturday of that week.
        assert_eq!(weekday(d + 5), 5);
        assert_eq!(weekday(d + 6), 6);
    }

    #[test]
    fn adding_days_crosses_a_leap_day_and_a_year_end() {
        assert_eq!(format(parse("2024-02-28").unwrap() + 2), "2024-03-01");
        assert_eq!(format(parse("2023-02-28").unwrap() + 1), "2023-03-01");
        assert_eq!(format(parse("2026-12-31").unwrap() + 1), "2027-01-01");
        assert_eq!(format(parse("2000-02-29").unwrap() + 1), "2000-03-01");
    }

    #[test]
    fn a_date_that_does_not_exist_is_refused() {
        for bad in [
            "2023-02-29",
            "2026-13-01",
            "2026-00-10",
            "2026-04-31",
            "2026-4-1",
            "26-04-01",
            "2026/04/01",
            "",
            "2026-04-0a",
            "2100-02-29",
        ] {
            assert_eq!(parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn dates_before_the_epoch_count_backwards() {
        assert_eq!(parse("1969-12-31"), Some(-1));
        assert_eq!(format(-1), "1969-12-31");
        assert_eq!(weekday(-1), 2);
    }

    #[test]
    fn the_monday_on_or_before_a_wednesday_is_two_days_earlier() {
        let wed = parse("2026-12-02").unwrap();
        assert_eq!(format(monday_on_or_before(wed)), "2026-11-30");
        assert_eq!(monday_on_or_before(monday_on_or_before(wed)), monday_on_or_before(wed));
    }
}
