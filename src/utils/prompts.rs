use crate::utils::{get_colors, get_unicode_string};

pub fn outro(message: &str) {
    let colors = get_colors();
    let s_bar = get_unicode_string("│", "|");
    let s_bar_end = get_unicode_string("└", "—");
    println!(
        "{}\n{}  {}\n",
        colors.gray(s_bar),
        colors.gray(s_bar_end),
        message
    );
}

pub fn intro(title: &str) {
    let colors = get_colors();
    let s_bar_start = get_unicode_string("┌", "T");
    println!("{}  {}\n", colors.gray(s_bar_start), title);
}
