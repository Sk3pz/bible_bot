// With 66 books, 1189 chapters, and ~31102 verses, and only 365 days in a year (sometimes 366),
// we need to schedule multiple verses / chapters per day to get through the entire Bible in a year.
// A simple approach is to create a reading schedule that assigns a set of chapters to each day.
// The scheduler should aim for 3 chapters a day, 4 on weekends.
// This schedule will leave us with a few days at the end of the year without any reading, but it will vary year to year.

use bible_lib::BibleLookup;
use chrono::NaiveDate;

pub struct Reading {
    pub start: BibleLookup,
    pub end: BibleLookup,
}

fn calculate_reading_for_day(date: &NaiveDate) -> Reading {
    // if it's saturday or sunday, read 4 chapters. Otherwise read 3.
    // 2024 is a leap year, so we have 366 days.
    

    todo!()
}