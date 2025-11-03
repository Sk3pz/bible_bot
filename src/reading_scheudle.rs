// With 66 books, 1189 chapters, and ~31102 verses, and only 365 days in a year (sometimes 366),
// we need to schedule multiple verses / chapters per day to get through the entire Bible in a year.
// A simple approach is to create a reading schedule that assigns a set of chapters to each day.
// The scheduler should aim for 3 chapters a day, 4 on weekends.
// This schedule will leave us with a few days at the end of the year without any reading, but it will vary year to year.

use bible_lib::{Bible, BibleLookup};
use chrono::{Datelike, NaiveDate, Weekday};

use crate::hey;

#[derive(Debug, Clone)]
pub struct Reading {
    pub start: BibleLookup,
    pub end: BibleLookup,
}

pub fn calculate_reading_for_day(date: &NaiveDate, bible: &Bible) -> Option<Reading> {
    // Weekend (Sat/Sun) → 4 chapters, otherwise 3
    let chapters_today = match date.weekday() {
        Weekday::Sat | Weekday::Sun => 4,
        _ => 3,
    };

    // Build a fully ordered list of (book, chapter) pairs
    let mut all_chapters = Vec::new();
    for book in bible.get_sorted_books() {
        if let Ok(mut chapters) = bible.get_chapters(&book) {
            chapters.sort();
            for chapter in chapters {
                all_chapters.push((book.clone(), chapter));
            }
        }
    }

    let total_chapters = all_chapters.len();

    // Count how many chapters were read on previous days
    let mut total_chapters_so_far = 0;
    for day_index in 1..date.ordinal() {
        let weekday = NaiveDate::from_yo_opt(date.year(), day_index)
            .unwrap()
            .weekday();
        total_chapters_so_far += match weekday {
            Weekday::Sat | Weekday::Sun => 4,
            _ => 3,
        };
    }

    // If we've already finished the Bible, stop
    if total_chapters_so_far >= total_chapters {
        return None;
    }

    // Determine range for today
    let start_index = total_chapters_so_far;
    let end_index = (start_index + chapters_today).min(total_chapters) - 1;

    let (start_book, start_chapter) = &all_chapters[start_index];
    let (end_book, end_chapter) = &all_chapters[end_index];

    let start = BibleLookup::new(start_book, *start_chapter, 1);
    let end = BibleLookup::new(
        end_book,
        *end_chapter,
        bible.get_max_verse(end_book, *end_chapter).unwrap_or(1),
    );

    Some(Reading { start, end })
}
