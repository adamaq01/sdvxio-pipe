use crate::glue::log_formatter_t;
use log::{Level, Log, Metadata, Record};
use std::ffi::CString;

#[derive(Debug)]
pub struct BT5Logger {
    pub(crate) misc: log_formatter_t,
    pub(crate) info: log_formatter_t,
    pub(crate) warning: log_formatter_t,
    pub(crate) fatal: log_formatter_t,
}

impl Log for BT5Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let logger = match record.level() {
            Level::Error => self.fatal,
            Level::Warn => self.warning,
            Level::Info => self.info,
            Level::Debug | Level::Trace => self.misc,
        };
        let message = CString::new(format!("{}", record.args())).unwrap();
        unsafe {
            if let Some(logger_fn) = logger {
                logger_fn(c"sdvxio-pipe".as_ptr(), message.as_ptr());
            }
        }
    }

    fn flush(&self) {
        // No-op
    }
}
