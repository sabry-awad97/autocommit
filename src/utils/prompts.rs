use colored::*;

use crate::utils::get_unicode_string;

struct BarColors {
    bar: Color,
    text: Color,
}

fn print_outro(message: &str, colors: &BarColors) {
    let s_bar = get_unicode_string("│", "|");
    let s_bar_end = get_unicode_string("└", "—");
    println!(
        "{}\n{} {}",
        s_bar.color(colors.bar),
        s_bar_end.color(colors.bar),
        message.color(colors.text)
    );
}

fn print_intro(title: &str, colors: &BarColors) {
    let s_bar_start = get_unicode_string("┌", "T");
    let title_len = title.chars().count();
    let max_len = s_bar_start.chars().count() + title_len;
    if max_len > 80 {
        panic!("Title is too long to fit within the bars.");
    }
    println!(
        "{}  {}",
        s_bar_start.color(colors.bar),
        title.color(colors.text)
    );
}

pub fn intro(title: &str) {
    let colors = BarColors {
        bar: Color::TrueColor {
            r: 128,
            g: 128,
            b: 128,
        },
        text: Color::White,
    };
    print_intro(title, &colors);
}

pub fn outro(title: &str) {
    let colors = BarColors {
        bar: Color::TrueColor {
            r: 128,
            g: 128,
            b: 128,
        },
        text: Color::White,
    };
    print_outro(title, &colors);
}
