use std::env;

use lazy_static::lazy_static;

fn is_unicode_supported() -> bool {
    if cfg!(windows) {
        env::var("CI").is_ok()
            || env::var("WT_SESSION").is_ok()
            || env::var("TERMINUS_SUBLIME").is_ok()
            || env::var("ConEmuTask")
                .map(|s| s == "{cmd::Cmder}")
                .unwrap_or(false)
            || env::var("TERM_PROGRAM")
                .map(|s| s == "Terminus-Sublime" || s == "vscode")
                .unwrap_or(false)
            || env::var("TERM")
                .map(|s| s == "xterm-256color" || s == "alacritty")
                .unwrap_or(false)
            || env::var("TERMINAL_EMULATOR")
                .map(|s| s == "JetBrains-JediTerm")
                .unwrap_or(false)
    } else {
        env::var("TERM").map(|s| s != "linux").unwrap_or(true)
    }
}

lazy_static! {
    static ref UNICODE_SUPPORTED: bool = is_unicode_supported();
}

pub fn get_unicode_string<'a>(unicode_string: &'a str, fallback_string: &'a str) -> &'a str {
    if *UNICODE_SUPPORTED {
        unicode_string
    } else {
        fallback_string
    }
}
