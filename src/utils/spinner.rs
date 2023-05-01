use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    thread,
    time::Duration,
};

use regex::Regex;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError, TrySendError};

use crate::utils::{get_unicode_string, get_colors};

#[derive(Debug, Clone)]
pub enum SpinnerMessage {
    Stop,
    Update(UpdateMessage),
}

#[derive(Debug, Clone)]
pub enum UpdateMessage {
    Message(String),
}

#[derive(Clone)]
pub struct Channel<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> Channel<T>
where
    T: Send + 'static + Clone,
{
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            sender: tx,
            receiver: rx,
        }
    }

    pub fn try_send(&self, message: T) -> Result<(), TrySendError<T>> {
        self.sender.try_send(message)
    }

    pub fn try_receive(&self) -> Result<T, TryRecvError> {
        self.receiver.try_recv()
    }
}

#[derive(Clone)]
pub struct SpinnerState {
    pub channel: Channel<SpinnerMessage>,
    pub current_index: usize,
    pub dots: String,
    pub frames: Vec<&'static str>,
    pub frame_duration: u64,
    pub n_dots: usize,
    pub trimmed_message: String,
    pub reverse: Arc<AtomicBool>,
}

impl SpinnerState {
    pub fn new(message: String) -> Self {
        let frames = vec!["◐", "◓", "◑", "◒"];
        let frame_duration = 50;

        let (trimmed_message, dots) = Self::trim_trailing_dots(message);
        let n_dots = dots.len();
        let reverse = Arc::new(AtomicBool::new(false));
        let channel = Channel::new();
        let current_index = 0;

        Self {
            current_index,
            dots,
            frames,
            frame_duration,
            n_dots,
            reverse,
            trimmed_message,
            channel,
        }
    }

    pub fn update(&mut self, message: UpdateMessage) {
        self.channel
            .try_send(SpinnerMessage::Update(message))
            .unwrap()
    }

    pub fn spin(&mut self, running: Arc<AtomicBool>, paused: Arc<(Mutex<bool>, Condvar)>) {
        let colors = get_colors();
        let output = io::stdout();
        let mut handle = output.lock();
        write!(handle, "\x1B[?25l").unwrap(); // hide cursor

        let s_bar = get_unicode_string("│", "|");
        println!("{}", colors.gray(s_bar));
        loop {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            let (lock, cvar) = &*paused;
            let mut paused = lock.lock().unwrap();
            while *paused {
                paused = cvar.wait(paused).unwrap();
            }

            let frames_length = self.frames.clone().len();
            let frames = self.frames.clone();
            let frame = frames[self.current_index % frames.len()].to_owned();
            let dots = ".".repeat(self.n_dots.min(self.dots.len()));

            let output_str = format!(
                "{} {}{}",
                colors.magenta(&frame),
                self.trimmed_message,
                colors.magenta(&dots)
            );
            write!(handle, "\r{}\x1B[K", &output_str).unwrap();
            self.current_index = match self.reverse.load(Ordering::SeqCst) {
                true => (self.current_index + frames_length - 1) % frames_length,
                false => (self.current_index + 1) % frames_length,
            };
            self.n_dots = (self.n_dots + 1) % (frames_length * 4);
            handle.flush().unwrap();

            thread::sleep(Duration::from_millis(self.frame_duration));

            let Ok(spin_message) = self.channel.try_receive() else { continue };
            match spin_message {
                SpinnerMessage::Stop => {
                    self.channel.try_send(SpinnerMessage::Stop).unwrap();
                }
                SpinnerMessage::Update(result) => match result {
                    UpdateMessage::Message(message) => {
                        let (message, dots) = Self::trim_trailing_dots(message.into());
                        self.trimmed_message = message;
                        self.dots = dots;
                    }
                },
            }
        }
    }
}

impl SpinnerState {
    fn trim_trailing_dots(s: String) -> (String, String) {
        let re = Regex::new(r"\.*$").unwrap();
        let message = re.replace(&s, "").to_owned();
        let message_dots = re
            .find(&s)
            .map(|m| m.as_str().to_owned())
            .unwrap_or_default();
        (message.to_string(), message_dots)
    }
}

pub struct Spinner {
    state: SpinnerState,
    running: Arc<AtomicBool>,
    paused: Arc<(Mutex<bool>, Condvar)>,
}

impl Spinner {
    pub fn new(message: impl Into<String>) -> Self {
        let state = SpinnerState::new(message.into());

        let running = Arc::new(AtomicBool::new(false));
        let paused = Arc::new((Mutex::new(false), Condvar::new()));
        Self {
            state,
            running,
            paused,
        }
    }

    pub fn start(&mut self, message: impl Into<String>) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }
        let running = self.running.clone();
        let paused = self.paused.clone();
        let mut state = self.state.clone();
        thread::spawn(move || state.spin(running, paused));
        self.running.store(true, Ordering::SeqCst);
        self.set_message(message);
    }

    pub fn stop(&mut self, message: impl Into<String>) {
        self.state.channel.try_send(SpinnerMessage::Stop).unwrap();
        self.running.store(false, Ordering::SeqCst);
        let s_bar = get_unicode_string("│", "|");
        let s_step_submit = get_unicode_string("◇", "o");
        let colors = get_colors();
        
        println!("\n{}", colors.gray(s_bar));
        println!("{} {}", colors.green(s_step_submit), message.into());
    }

    pub fn set_message<T>(&mut self, message: T)
    where
        T: Into<String>,
    {
        self.state.update(UpdateMessage::Message(message.into()))
    }
}

pub fn spinner() -> Spinner {
    Spinner::new("")
}
