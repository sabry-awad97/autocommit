use colored::{Color, Colorize};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::utils::get_unicode_string;

pub struct Spinner {
    pb: ProgressBar,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(50));
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("◐◓◑◒")
                .template("{spinner:.magenta} {msg}")
                .unwrap(),
        );
        Self { pb }
    }

    pub fn start(&mut self, message: &str) {
        self.pb.set_message(message.to_string());
    }

    pub fn stop(&mut self, message: &str) {
        let s_bar = get_unicode_string("│", "|").color(Color::TrueColor {
            r: 128,
            g: 128,
            b: 128,
        });

        self.pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("◇◇")
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        println!("{}", s_bar);
        self.pb.finish_with_message(message.to_string());
    }
}

pub fn spinner() -> Spinner {
    Spinner::new("")
}
