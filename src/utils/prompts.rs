use colored::*;

use crate::utils::get_unicode_string;

pub fn outro(message: &str) {
    let s_bar = get_unicode_string("│", "|");
    let s_bar_end = get_unicode_string("└", "—");
    println!(
        "{}\n{} {}",
        s_bar.color(Color::TrueColor {
            r: 128,
            g: 128,
            b: 128
        }),
        s_bar_end.color(Color::TrueColor {
            r: 128,
            g: 128,
            b: 128
        }),
        message
    );
}

pub fn intro(title: &str) {
    let s_bar_start = get_unicode_string("┌", "T");
    println!(
        "{}  {}",
        s_bar_start.color(Color::TrueColor {
            r: 128,
            g: 128,
            b: 128
        }),
        title
    );
}
