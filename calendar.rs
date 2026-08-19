use chrono::{NaiveDate, Datelike, Local, Weekday, Duration};

#[derive(Debug)]
pub struct BankHoliday {
    pub date: NaiveDate,
    pub name: String,
}

pub fn is_rochdale_term_time(date: NaiveDate) -> bool {
    let year = date.year();

    let term_dates = [
        (
            NaiveDate::from_ymd_opt(year, 1, 5),
            NaiveDate::from_ymd_opt(year, 2, 13),
        ),
        (
            NaiveDate::from_ymd_opt(year, 2, 23),
            NaiveDate::from_ymd_opt(year, 3, 27),
        ),
        (
            NaiveDate::from_ymd_opt(year, 4, 13),
            NaiveDate::from_ymd_opt(year, 5, 22),
        ),
        (
            NaiveDate::from_ymd_opt(year, 6, 1),
            NaiveDate::from_ymd_opt(year, 7, 21),
        ),
        (
            NaiveDate::from_ymd_opt(year, 9, 2),
            NaiveDate::from_ymd_opt(year, 10, 23),
        ),
        (
            NaiveDate::from_ymd_opt(year, 11, 3),
            NaiveDate::from_ymd_opt(year, 12, 19),
        ),
    ];

    term_dates.iter().any(|(start, end)| {
        match(start, end) {
            (Some(start), Some(end)) => date >= *start && date <= *end,
            _=> false,
        }
    })
}

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}


pub fn retention_cutoff(today: NaiveDate) -> NaiveDate {
    today - Duration::days(1825)
}


pub fn generate_month(year: i32, month: u32) -> Vec<NaiveDate> {
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    let mut dates = Vec::new();
    let mut current_date = first_day;

    while current_date.month() == month {
        dates.push(current_date);
        current_date += Duration::days(1);
    }

    dates
}


pub fn is_weekend(date: NaiveDate) -> bool {
    match date.weekday() {
        Weekday::Sat | Weekday::Sun => true,
        _ => false,
    }
}

pub fn is_bank_holiday(
    date: NaiveDate,
    bank_holidays: &[BankHoliday],
) -> bool {
    bank_holidays
        .iter()
        .any(|holiday| holiday.date == date)
}

pub fn uk_bank_holidays(year: i32) -> Vec<BankHoliday> {
    let easter = easter_sunday(year);

    let new_years_day =
        NaiveDate::from_ymd_opt(year, 1, 1).unwrap();

    let christmas_day =
        NaiveDate::from_ymd_opt(year, 12, 25).unwrap();

    let boxing_day =
        NaiveDate::from_ymd_opt(year, 12, 26).unwrap();

    let mut holidays = vec![
        BankHoliday {
            date: new_years_day,
            name: "New Year's Day".to_string(),
        },

        BankHoliday {
            date: easter - Duration::days(2),
            name: "Good Friday".to_string(),
        },

        BankHoliday {
            date: easter + Duration::days(1),
            name: "Easter Monday".to_string(),
        },

        BankHoliday {
            date: first_monday_of_may(year),
            name: "Early May Bank Holiday".to_string(),
        },

        BankHoliday {
            date: last_monday_of_may(year),
            name: "Spring Bank Holiday".to_string(),
        },

        BankHoliday {
            date: last_monday_of_august(year),
            name: "Summer Bank Holiday".to_string(),
        },

        BankHoliday {
            date: christmas_day,
            name: "Christmas Day".to_string(),
        },

        BankHoliday {
            date: boxing_day,
            name: "Boxing Day".to_string(),
        },
    ];

    // New Year's Day substitute
    add_single_substitute_day(
        &mut holidays,
        new_years_day,
        "New Year's Day",
    );

    // Christmas and Boxing Day substitutes
    add_christmas_substitute_days(
        &mut holidays,
        christmas_day,
        boxing_day,
    );

    holidays
}



fn easter_sunday(year: i32) -> NaiveDate {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;

    let month = (h + l - 7 * m + 114) / 31;
    let day = ((h + l - 7 * m + 114) % 31) + 1;

    NaiveDate::from_ymd_opt(
        year,
        month as u32,
        day as u32,
    )
    .unwrap()
}

fn first_monday_of_may(year: i32) -> NaiveDate {
    let mut date =
        NaiveDate::from_ymd_opt(year, 5, 1).unwrap();

    while date.weekday() != Weekday::Mon {
        date += Duration::days(1);
    }

    date
}


fn last_monday_of_may(year: i32) -> NaiveDate {
    let mut date =
        NaiveDate::from_ymd_opt(year, 5, 31).unwrap();

    while date.weekday() != Weekday::Mon {
        date -= Duration::days(1);
    }

    date
}


fn last_monday_of_august(year: i32) -> NaiveDate {
    let mut date =
        NaiveDate::from_ymd_opt(year, 8, 31).unwrap();

    while date.weekday() != Weekday::Mon {
        date -= Duration::days(1);
    }

    date
}



fn add_single_substitute_day(
    holidays: &mut Vec<BankHoliday>,
    date: NaiveDate,
    name: &str,
) {
    match date.weekday() {
        Weekday::Sat => {
            holidays.push(BankHoliday {
                date: date + Duration::days(2),
                name: format!("{} (substitute)", name),
            });
        }

        Weekday::Sun => {
            holidays.push(BankHoliday {
                date: date + Duration::days(1),
                name: format!("{} (substitute)", name),
            });
        }

        _ => {}
    }
}


fn add_christmas_substitute_days(
    holidays: &mut Vec<BankHoliday>,
    christmas_day: NaiveDate,
    boxing_day: NaiveDate,
) {
    if christmas_day.weekday() == Weekday::Sat
        && boxing_day.weekday() == Weekday::Sun
    {
        holidays.push(BankHoliday {
            date: christmas_day + Duration::days(2),
            name: "Christmas Day (substitute)".to_string(),
        });

        holidays.push(BankHoliday {
            date: boxing_day + Duration::days(2),
            name: "Boxing Day (substitute)".to_string(),
        });
    } else {
        add_single_substitute_day(
            holidays,
            christmas_day,
            "Christmas Day",
        );

        add_single_substitute_day(
            holidays,
            boxing_day,
            "Boxing Day",
        );
    }
}



