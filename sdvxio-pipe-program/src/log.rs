use anstyle::Style;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

pub use log::*;

#[derive(Debug)]
pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            file: File::create("sdvxio-pipe.log").unwrap(),
        }
    }

    pub fn init(self) {
        env_logger::builder()
            .filter_level(LevelFilter::Trace)
            .filter_module(
                "sdvxio_pipe_program",
                if cfg!(debug_assertions) {
                    LevelFilter::Trace
                } else {
                    LevelFilter::Info
                },
            )
            .parse_default_env()
            .target(env_logger::Target::Pipe(Box::new(self)))
            .format(|f, record| {
                use crate::log::{colored_level, max_target_width, Padded};
                use std::io::Write;

                let target = record.target();
                let max_width = max_target_width(target);

                let style = f.default_level_style(record.level());
                let level = colored_level(style, record.level());

                let style = f.default_level_style(record.level()).bold();
                let target = Padded {
                    value: target,
                    width: max_width,
                }
                .styled(style);

                let time = chrono::Local::now().format("%d/%m/%Y %H:%M:%S");

                writeln!(f, "[{}] {} {} -> {}", time, level, target, record.args())
            })
            .init();
    }
}

impl Write for Logger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

pub(crate) struct Styled<T> {
    pub(crate) style: Style,
    pub(crate) item: T,
}

impl<T: fmt::Display> fmt::Display for Styled<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}{:#}", self.style, self.item, self.style)
    }
}

pub(crate) trait ToStyled<T> {
    fn styled(self, style: Style) -> Styled<T>;
}

impl<T> ToStyled<T> for T {
    fn styled(self, style: Style) -> Styled<T> {
        Styled { style, item: self }
    }
}

pub(crate) struct Padded<T> {
    pub(crate) value: T,
    pub(crate) width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}

pub(crate) static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

pub(crate) fn colored_level(style: Style, level: Level) -> Styled<&'static str> {
    match level {
        Level::Trace => "TRACE".styled(style.fg_color(Some(anstyle::AnsiColor::Magenta.into()))),
        Level::Debug => "DEBUG".styled(style.fg_color(Some(anstyle::AnsiColor::Blue.into()))),
        Level::Info => " INFO".styled(style.fg_color(Some(anstyle::AnsiColor::Green.into()))),
        Level::Warn => " WARN".styled(style.fg_color(Some(anstyle::AnsiColor::Yellow.into()))),
        Level::Error => "ERROR".styled(style.fg_color(Some(anstyle::AnsiColor::Red.into()))),
    }
}
