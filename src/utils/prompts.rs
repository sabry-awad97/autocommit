use crate::utils::{get_unicode_string, Colors};

pub fn outro(message: &str) {
    let s_bar = get_unicode_string("│", "|");
    let s_bar_end = get_unicode_string("└", "—");
    println!(
        "{}\n{}  {}\n",
        Colors.gray(s_bar),
        Colors.gray(s_bar_end),
        message
    );
}

pub fn intro(title: &str) {
    let s_bar_start = get_unicode_string("┌", "T");
    println!("{}  {}\n", Colors.gray(s_bar_start), title);
}
