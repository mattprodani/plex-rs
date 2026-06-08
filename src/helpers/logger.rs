use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use core::sync::atomic::{AtomicBool, Ordering};
use log::{Metadata, Record};
use spin::Mutex;
use uefi::boot;
use uefi::fs::FileSystem;
use uefi::CString16;

pub struct FileLogger {
    initialized: AtomicBool,
    enabled: AtomicBool,
    buffer: Mutex<String>,
    path: &'static str,
}

impl FileLogger {
    #[must_use]
    pub const fn new(path: &'static str) -> Self {
        Self {
            initialized: AtomicBool::new(false),
            enabled: AtomicBool::new(false),
            buffer: Mutex::new(String::new()),
            path,
        }
    }

    pub fn init(&'static self) {
        if self
            .initialized
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        log::set_logger(self).ok();
        log::set_max_level(log::STATIC_MAX_LEVEL);
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::SeqCst);
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }

    fn append(&self, record: &Record) {
        let file = record.file().unwrap_or("<unknown>");
        let line = record.line().unwrap_or(0);
        let mut buf = self.buffer.lock();
        let level = record.level();
        let _ = writeln!(*buf, "[{level:>5}] {file}:{line} {}", record.args());
    }

    fn flush_to_file(&self) {
        let Ok(path) = CString16::try_from(self.path) else {
            return;
        };
        let contents: Vec<u8> = {
            let buf = self.buffer.lock();
            if buf.is_empty() {
                return;
            }
            buf.as_bytes().to_vec()
        };
        let mut fs = match boot::get_image_file_system(boot::image_handle()) {
            Ok(fs) => FileSystem::new(fs),
            Err(_) => return,
        };
        if let Err(err) = fs.write(path.as_ref(), contents) {
            if !WRITE_ERROR_REPORTED.swap(true, Ordering::SeqCst) {
                uefi::println!("file logger write failed: {err:?}");
            }
        } else {
            uefi::println!("file logger success");
        }
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled.load(Ordering::SeqCst) {
            return;
        }
        if !self.enabled(record.metadata()) {
            return;
        }

        self.append(record);
        self.flush_to_file();
    }

    fn flush(&self) {
        self.flush_to_file();
    }
}

static FILE_LOGGER: FileLogger = FileLogger::new("\\plex.log");
static WRITE_ERROR_REPORTED: AtomicBool = AtomicBool::new(false);

pub fn init_file_logger() {
    FILE_LOGGER.init();
    FILE_LOGGER.enable();
}

pub fn disable_file_logger() {
    FILE_LOGGER.disable();
}
