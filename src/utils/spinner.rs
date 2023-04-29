use spinergy::{Color, Spinner, SpinnerConfig, SpinnerStyle};

use crate::utils::{get_unicode_string, Colors};

pub fn spinner(message: &str) -> Spinner {
    let s_bar = get_unicode_string("│", "|");
    println!("{}", Colors.gray(s_bar));
    let config = SpinnerConfig::new(SpinnerStyle::CircleHalves, message)
        .with_style_color(Color::Magenta)
        .build();
    Spinner::new(config)
}
