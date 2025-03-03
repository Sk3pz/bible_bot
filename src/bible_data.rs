use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use rand::prelude::SliceRandom;

#[derive(Clone, Debug)]
pub struct BibleLookup {
    pub book: String,
    pub chapter: u32,
    pub verse: u32,
    pub thru_verse: Option<u32>,
}

impl BibleLookup {
    pub fn new<S: Into<String>>(book: S, chapter: u32, verse: u32) -> Self {
        let book = book.into();
        let book = book.to_lowercase();
        Self {
            book,
            chapter,
            verse,
            thru_verse: None,
        }
    }

    pub fn new_range<S: Into<String>>(book: S, chapter: u32, verse: u32, thru_verse: u32) -> Self {
        let book = book.into();
        let book = book.to_lowercase();
        Self {
            book,
            chapter,
            verse,
            thru_verse: Some(thru_verse),
        }
    }

    // Capitalize the book for display
    pub fn capitalize_book(name: &String) -> String {
        // Split the input string by whitespace into words
        name.split_whitespace()
            // For each word, apply the following transformation
            .map(|word| {
                // Convert the word into characters
                let mut chars = word.chars();
                // If there's a first character, convert it to uppercase and concatenate it with the rest of the characters
                if let Some(first_char) = chars.next() {
                    if first_char.is_numeric() {
                        // If the first character is numeric, leave it unchanged
                        first_char.to_string() + &chars.collect::<String>()
                    } else {
                        // If the first character is not numeric, capitalize it
                        first_char.to_uppercase().chain(chars).collect::<String>()
                    }
                } else {
                    // If the word is empty, return an empty string
                    String::new()
                }
            })
            // Collect the transformed words back into a single string, separated by whitespace
            .collect::<Vec<String>>().join(" ")
    }
}

impl Display for BibleLookup {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(thru_verse) = self.thru_verse {
            write!(f, "{} {}:{}-{}", BibleLookup::capitalize_book(&self.book), self.chapter, self.verse, thru_verse)
        } else {
            write!(f, "{} {}:{}", BibleLookup::capitalize_book(&self.book), self.chapter, self.verse)
        }
    }
}

pub struct Bible {
    pub verses: HashMap<String /* Book */,
                HashMap<u32 /* Chapter */,
                HashMap<u32 /* Verse */, String /* Text */>>>,
}

impl Bible {
    // assumes the translation is in format "Book Chapter:Verse Text" per line in the file
    pub fn parse<P: Into<String>>(path: P) -> Self {
        let path = path.into();
        let lines = std::fs::read_to_string(path.clone()).unwrap();
        let mut verses = HashMap::new();

        for line in lines.lines() {
            // to handle cases like `1 samuel` and `Song of Solomon`, split by ':' first and then split by whitespace
            let mut parts = line.split(':');
            // split the first part by whitespace
            let book_chapter = parts.next().unwrap().split_whitespace();
            let count = book_chapter.clone().count();
            let chapter = book_chapter.clone().last().unwrap().parse::<u32>().unwrap();
            let book = book_chapter.take(count - 1).collect::<Vec<&str>>().join(" ").to_lowercase();

            let verse_text = parts.next().unwrap().split_whitespace();
            let verse = verse_text.clone().next().unwrap().parse::<u32>().unwrap();
            let text = verse_text.clone().skip(1).collect::<Vec<&str>>().join(" ");

            if !verses.contains_key(&book) {
                verses.insert(book.to_string(), HashMap::new());
            }
            if !verses.get_mut(&book).unwrap().contains_key(&chapter) {
                verses.get_mut(&book).unwrap().insert(chapter, HashMap::new());
            }
            verses.get_mut(&book).unwrap().get_mut(&chapter).unwrap().insert(verse, text.to_string());
        }

        Self {
            verses,
        }
    }

    fn replace_superscript(s: String) -> String {
        s.chars().map(|c| {
            match c {
                '0' => '⁰',
                '1' => '¹',
                '2' => '²',
                '3' => '³',
                '4' => '⁴',
                '5' => '⁵',
                '6' => '⁶',
                '7' => '⁷',
                '8' => '⁸',
                '9' => '⁹',
                _ => c,
            }
        }).collect()
    }

    pub fn get_verse(&self, lookup: BibleLookup) -> Option<String> {
        if let Some(thru_verse) = lookup.thru_verse {
            let mut verse_text = String::new();
            for verse in lookup.verse..=thru_verse {
                if let Some(text) = self.verses.get(&lookup.book)?.get(&lookup.chapter)?.get(&verse) {
                    verse_text.push_str(&format!("{}{} ", Self::replace_superscript(verse.to_string()), text));
                }
            }
            return Some(verse_text.trim().to_string());
        }
        self.verses.get(&lookup.book)?.get(&lookup.chapter)?.get(&lookup.verse).map(|s| s.clone())
    }

    pub fn get_chapter(&self, book: &str, chapter: u32) -> Option<String> {
        let mut chapter_text = String::new();
        // sort the verses by verse number
        let mut verses = self.verses.get(book)?.get(&chapter)?.iter().collect::<Vec<(&u32, &String)>>();
        verses.sort_by(|a, b| a.0.cmp(b.0));
        for (verse, text) in verses {
            let verse_designation = Self::replace_superscript(verse.to_string());
            chapter_text.push_str(&format!("{}{} ", verse_designation, text));
        }
        Some(chapter_text)
    }

    pub fn get_books(&self) -> Vec<String> {
        self.verses.keys().map(|s| s.to_string()).collect()
    }

    pub fn get_chapters(&self, book: &str) -> Option<Vec<u32>> {
        self.verses.get(book).map(|chapters| chapters.keys().map(|c| *c).collect())
    }

    pub fn get_verses(&self, book: &str, chapter: u32) -> Option<Vec<u32>> {
        self.verses.get(book).map(|chapters| chapters.get(&chapter).map(|verses| verses.keys().map(|v| *v).collect())).flatten()
    }

    pub fn get_max_verse(&self, book: &str, chapter: u32) -> Option<u32> {
        self.verses.get(book).map(|chapters| chapters.get(&chapter).map(|verses| *verses.keys().max().unwrap())).flatten()
    }

    pub fn random_verse(&self) -> BibleLookup {
        let books = self.get_books();
        let book = books.choose(&mut rand::thread_rng()).unwrap();
        let chapters = self.get_chapters(book).unwrap();
        let chapter = chapters.choose(&mut rand::thread_rng()).unwrap();
        let verses = self.get_verses(book, *chapter).unwrap();
        let verse = verses.choose(&mut rand::thread_rng()).unwrap();
        BibleLookup::new(book.clone(), *chapter, *verse)
    }
}