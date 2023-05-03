use colored::{Color, Colorize};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

use crate::utils::get_unicode_string;

pub struct Spinner {
    pb: ProgressBar,
    start_time: Instant,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(50));
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("◐◓◑◒")
                .template("{spinner:.magenta} {msg} {elapsed}")
                .unwrap(),
        );
        Self {
            pb,
            start_time: Instant::now(),
        }
    }

    pub fn start(&mut self, message: &str) {
        self.pb.set_message(message.to_string());
        self.start_time = Instant::now();
    }

    pub fn stop(&mut self, message: &str) {
        let elapsed = self.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs();
        let elapsed_millis = elapsed.subsec_millis();
        let elapsed_str = if elapsed_secs > 0 {
            format!("{}s {}ms", elapsed_secs, elapsed_millis)
        } else {
            format!("{}ms", elapsed_millis)
        };

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
        self.pb.finish_with_message(format!("{} (elapsed time: {})", message, elapsed_str));
    }
}

pub fn spinner() -> Spinner {
    Spinner::new("")
}
