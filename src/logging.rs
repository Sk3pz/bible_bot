use core::fmt;

use better_term::{Color, flush_styles};

fn raw_log(prefix: String, msg_color: Color, args: fmt::Arguments) {
    println!(
        "{b}[{}{b}] {}{}",
        //"{} {}{}",
        prefix,
        msg_color,
        args,
        b = Color::BrightBlack
    );
    flush_styles();
}

pub fn _wow(args: fmt::Arguments) {
    println!();
    raw_log(format!("{}$", Color::BrightPurple), Color::BrightPurple, args);
    println!();
}

#[macro_export]
macro_rules! wow {
    ($($arg:tt)*) => { $crate::logging::_wow(format_args!($($arg)*)) }
}

pub fn _say(args: fmt::Arguments) {
    raw_log(format!("{}#", Color::White), Color::BrightWhite, args);
}

#[macro_export]
macro_rules! say {
    ($($arg:tt)*) => { $crate::logging::_say(format_args!($($arg)*)) }
}

pub fn _yay(args: fmt::Arguments) {
    raw_log(format!("{}✔", Color::BrightGreen), Color::BrightGreen, args);
}

#[macro_export]
macro_rules! yay {
    ($($arg:tt)*) => { $crate::logging::_yay(format_args!($($arg)*)) }
}

pub fn _hey(args: fmt::Arguments) {
    raw_log(format!("{}!", Color::Yellow), Color::BrightYellow, args);
}

#[macro_export]
macro_rules! hey {
    ($($arg:tt)*) => { $crate::logging::_hey(format_args!($($arg)*)) }
}

pub fn _nay(args: fmt::Arguments) {
    raw_log(format!("{}✘", Color::Red), Color::BrightRed, args);
}

#[macro_export]
macro_rules! nay {
    ($($arg:tt)*) => { $crate::logging::_nay(format_args!($($arg)*)) }
}