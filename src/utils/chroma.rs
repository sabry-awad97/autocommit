use atty::Stream;
use std::env;
use structopt::lazy_static::lazy_static;

pub struct Chroma {
    is_color_supported: bool,
}

impl Chroma {
    fn new() -> Self {
        let is_color_supported = match env::var("NO_COLOR") {
            Ok(_) => false,
            Err(_) => match env::var("FORCE_COLOR") {
                Ok(_) => true,
                Err(_) => {
                    atty::is(Stream::Stdout) && env::var("TERM").unwrap_or_default() != "dumb"
                }
            },
        };
        Self { is_color_supported }
    }

    fn formatter<'a>(
        &self,
        open: &'a str,
        close: &'a str,
        replace: Option<&'a str>,
    ) -> impl Fn(&'a str) -> String + 'a {
        move |input| {
            let mut string = input.to_string();
            if let Some(index) = string.find(close) {
                let replaced = replace.unwrap_or(close);
                string.replace_range(index..index + close.len(), replaced);
            }
            format!("{}{}{}", open, string, close)
        }
    }

    fn create_colors(&self) -> Chroma {
        let enabled = self.is_color_supported;
        Chroma {
            is_color_supported: enabled,
        }
    }

    fn reset(&self, s: &str) -> String {
        if self.is_color_supported {
            format!("\x1b[0m{}{}\x1b[0m", s, "\x1b[0m")
        } else {
            s.to_string()
        }
    }

    fn bold(&self, s: &str) -> String {
        self.formatter("\x1b[1m", "\x1b[22m", Some("\x1b[22m\x1b[1m"))(s)
    }

    fn dim(&self, s: &str) -> String {
        self.formatter("\x1b[2m", "\x1b[22m", Some("\x1b[22m\x1b[2m"))(s)
    }

    fn italic(&self, s: &str) -> String {
        self.formatter("\x1b[3m", "\x1b[23m", None)(s)
    }

    fn underline(&self, s: &str) -> String {
        self.formatter("\x1b[4m", "\x1b[24m", None)(s)
    }

    fn inverse(&self, s: &str) -> String {
        self.formatter("\x1b[7m", "\x1b[27m", None)(s)
    }

    fn hidden(&self, s: &str) -> String {
        self.formatter("\x1b[8m", "\x1b[28m", None)(s)
    }

    fn strikethrough(&self, s: &str) -> String {
        self.formatter("\x1b[9m", "\x1b[29m", None)(s)
    }

    fn black(&self, s: &str) -> String {
        self.formatter("\x1b[30m", "\x1b[39m", None)(s)
    }

    fn red(&self, s: &str) -> String {
        self.formatter("\x1b[31m", "\x1b[39m", None)(s)
    }

    fn green(&self, s: &str) -> String {
        self.formatter("\x1b[32m", "\x1b[39m", None)(s)
    }

    fn yellow(&self, s: &str) -> String {
        self.formatter("\x1b[33m", "\x1b[39m", None)(s)
    }

    fn blue(&self, s: &str) -> String {
        self.formatter("\x1b[34m", "\x1b[39m", None)(s)
    }

    fn magenta(&self, s: &str) -> String {
        self.formatter("\x1b[35m", "\x1b[39m", None)(s)
    }

    fn cyan(&self, s: &str) -> String {
        self.formatter("\x1b[36m", "\x1b[39m", None)(s)
    }

    fn white(&self, s: &str) -> String {
        self.formatter("\x1b[37m", "\x1b[39m", None)(s)
    }

    pub fn gray(&self, s: &str) -> String {
        self.formatter("\x1b[90m", "\x1b[39m", None)(s)
    }

    fn bg_black(&self, s: &str) -> String {
        self.formatter("\x1b[40m", "\x1b[49m", None)(s)
    }

    fn bg_red(&self, s: &str) -> String {
        self.formatter("\x1b[41m", "\x1b[49m", None)(s)
    }

    fn bg_green(&self, s: &str) -> String {
        self.formatter("\x1b[42m", "\x1b[49m", None)(s)
    }

    fn bg_yellow(&self, s: &str) -> String {
        self.formatter("\x1b[43m", "\x1b[49m", None)(s)
    }

    fn bg_blue(&self, s: &str) -> String {
        self.formatter("\x1b[44m", "\x1b[49m", None)(s)
    }

    fn bg_magenta(&self, s: &str) -> String {
        self.formatter("\x1b[45m", "\x1b[49m", None)(s)
    }

    fn bg_cyan(&self, s: &str) -> String {
        self.formatter("\x1b[46m", "\x1b[49m", None)(s)
    }

    fn bg_white(&self, s: &str) -> String {
        self.formatter("\x1b[47m", "\x1b[49m", None)(s)
    }
}

lazy_static! {
    pub static ref CHROMA: Chroma = Chroma::new();
}

pub use CHROMA as Colors;
