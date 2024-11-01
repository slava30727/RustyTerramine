#![allow(clippy::manual_strip, clippy::too_many_arguments)]

use crate::{
    prelude::*,
    concurrency::channel::Channel,
};



module_constructor! {
    env_logger::init();

    // * Safety
    // * 
    // * Safe, because it's going on in module
    // * constructor, so no one access the update list.
    unsafe {
        app::update::push_function_lock_free(update);
    }
}



lazy_static! {
    static ref CHANNEL: Mutex<Channel<Message>> = Mutex::new(Channel::default());
}

static LOG_MESSAGES: Mutex<VecDeque<Message>> = Mutex::new(VecDeque::new());



#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Display)]
#[display("[{msg_type}]-[{from}]: {content}")]
pub struct Message {
    pub content: StaticStr,
    pub from: StaticStr,
    pub msg_type: MsgType,
}
assert_impl_all!(Message: Send, Sync);



#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Display)]
#[display(style = "lowercase")]
pub enum MsgType {
    #[default]
    Info,
    Error,
}



pub fn recv_all() {
    let mut channel = CHANNEL.lock();
    let mut messages = LOG_MESSAGES.lock();

    while let Ok(msg) = channel.receiver.try_recv() {
        messages.push_back(msg);
    }
}

pub fn update() {
    recv_all();
}

pub fn log_impl(msg_type: MsgType, from: impl Into<StaticStr>, content: impl Into<StaticStr>) {
    let (from, content) = (from.into(), content.into());

    let message = Message { msg_type, from, content};

    eprintln!("{message}");
    CHANNEL.lock()
        .sender
        .send(message)
        .expect("failed to send message");
}

pub fn scope_impl(from: impl Into<StaticStr>, work: impl Into<StaticStr>) -> LogGuard {
    LogGuard::new(from, work)
}



#[must_use]
#[derive(Debug)]
pub struct LogGuard {
    pub from: StaticStr,
    pub work: StaticStr,
}
assert_impl_all!(LogGuard: Send, Sync);

impl LogGuard {
    pub fn new(from: impl Into<StaticStr>, work: impl Into<StaticStr>) -> Self {
        let (from, work) = (from.into(), work.into());
        info!(from = from.clone(), "start {work}.");
        Self { from, work }
    }
}

impl Drop for LogGuard {
    fn drop(&mut self) {
        let from = mem::take(&mut self.from);
        info!(from = from, "end {work}.", work = self.work);
    }
}



#[macro_export]
macro_rules! log {
    ($msg_type:ident, from = $from:expr, $($content:tt)*) => {{
        use $crate::app::utils::logger::{log_impl, MsgType::$msg_type};
        log_impl($msg_type, $from, std::fmt::format(format_args!($($content)*)));
    }};
}

#[macro_export]
macro_rules! info {
    (from = $from:expr, $($fmt:tt)*) => {
        $crate::logger::log!(Info, from = $from, $($fmt)*);
    };

    ($($fmt:tt)*) => { $crate::logger::info!(from = "*unknown*", $($fmt)*); };
}

#[macro_export]
macro_rules! error {
    (from = $from:expr, $($fmt:tt)*) => {
        $crate::logger::log!(Error, from = $from, $($fmt)*)
    };

    ($($fmt:tt)*) => { $crate::logger::error!(from = "*unknown*", $($fmt)*); };
}

#[macro_export]
macro_rules! log_dbg {
    ($expr:expr) => {{
        use $crate::app::utils::logger::log;
        let result = $expr;
        info!(from = "dbg", "{expr} = {result:?}", expr = stringify!($expr));
        result
    }};
}

#[macro_export]
macro_rules! log_scope {
    (from = $from:expr, $($content:tt)*) => {
        let _logger_scope_guard = $crate::app::utils::logger::scope_impl(
            $from, std::fmt::format(format_args!($($content)*))
        );
    };
}

pub use {log, info, error, log_dbg, log_scope};


pub fn build_window(ui: &mut egui::Ui) {
    static INPUT_HISTORY: Mutex<Vec<String>> = const_default();
    let mut input_history = INPUT_HISTORY.lock();

    static INPUT: Mutex<String> = const_default();
    let mut input = INPUT.lock();

    macros::atomic_static! {
        static LAST_SEARCH_INDEX: usize = usize::MAX;
    }
    
    let text_edit_response = ui.add(
        egui::TextEdit::singleline(input.deref_mut())
            .cursor_at_end(true)
            .hint_text("Input a command here...")
            .desired_width(f32::INFINITY)
            .font(egui::TextStyle::Monospace)
    );

    if text_edit_response.lost_focus() {
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            error!(from = "logger", "command interpreter is detached");

            LAST_SEARCH_INDEX.store(0, Release);
        }

        input_history.push(mem::take(input.deref_mut()));
    } else if !input_history.is_empty() && text_edit_response.has_focus() {
        let mut index = LAST_SEARCH_INDEX.load(Acquire);

        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            match index {
                0 | usize::MAX => index = input_history.len() - 1,
                _ => index -= 1,
            }

            match input_history.get(index) {
                None => input.clear(),
                Some(src) => input.clone_from(src),
            }
        } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            if index < input_history.len() - 1 {
                index += 1;
                input.clone_from(&input_history[index]);
            } else {
                index = usize::MAX;
                input.clear();
            }
        }

        LAST_SEARCH_INDEX.store(index, Release);
    }

    for msg in LOG_MESSAGES.lock().iter().rev() {
        let color = match msg.msg_type {
            MsgType::Error => egui::Color32::RED,
            MsgType::Info  => egui::Color32::GRAY,
        };

        ui.label(egui::RichText::new(msg.to_string())
            .color(color)
            .monospace()
        );
    }
}



pub trait LogError<T> {
    fn log_error(self, from: impl Into<StaticStr>, msg: impl Into<StaticStr>) -> T where T: Default;
    fn log_error_or(self, from: impl Into<StaticStr>, msg: impl Into<StaticStr>, default: T) -> T;
    fn log_error_or_else(self, from: impl Into<StaticStr>, msg: impl Into<StaticStr>, f: impl FnOnce() -> T) -> T;
}

impl<T, E: std::fmt::Display> LogError<T> for Result<T, E> {
    fn log_error(self, from: impl Into<StaticStr>, msg: impl Into<StaticStr>) -> T where T: Default {
        match self {
            Ok(item) => item,
            Err(err) => {
                let msg: StaticStr = msg.into();
                log!(Error, from = from, "{msg}: {err}");
                default()
            }
        }
    }

    fn log_error_or(self, from: impl Into<StaticStr>, msg: impl Into<StaticStr>, default: T) -> T {
        match self {
            Ok(item) => item,
            Err(err) => {
                let msg = msg.into();
                log!(Error, from = from, "{msg}: {err}");
                default
            }
        }
    }

    fn log_error_or_else(self, from: impl Into<StaticStr>, msg: impl Into<StaticStr>, f: impl FnOnce() -> T) -> T {
        match self {
            Ok(item) => item,
            Err(err) => {
                let msg = msg.into();
                log!(Error, from = from, "{msg}: {err}");
                f()
            }
        }
    }
}