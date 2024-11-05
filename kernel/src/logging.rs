//! Temporary implementation of kernel log

use core::fmt::Write;
use log::{Metadata, Record};

/// Global instance of the kernel logger.
static LOGGER: KernelLog = KernelLog {};

#[expect(clippy::unwrap_used, reason = "This is not a final implementation")]
pub fn initialize_kernel_log() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
}

struct KernelLog;

impl log::Log for KernelLog {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut writer = DebugWriter {};
            writeln!(&mut writer, "{}", record.args()).unwrap();
        }
    }

    fn flush(&self) {}
}

struct DebugWriter;

impl core::fmt::Write for DebugWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            crate::arch::io::Port(0xe9).write_u8(c);
        }
        Ok(())
    }
}
