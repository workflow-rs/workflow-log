use cfg_if::cfg_if;
use std::fmt;

cfg_if! {
    if #[cfg(target_arch = "bpf")] {
        pub use workflow_log::levels::{ Level, LevelFilter };
    } else {
        pub use log::{ Level, LevelFilter };
    }
}

cfg_if! {
    if #[cfg(any(target_arch = "wasm32", target_arch = "bpf"))] {
        static mut LEVEL_FILTER : LevelFilter = LevelFilter::Trace;
        #[inline(always)]
        pub fn log_level_enabled(level: Level) -> bool { 
            unsafe { LEVEL_FILTER >= level } 
        }
        pub fn set_log_level(level: LevelFilter) { 
            unsafe { LEVEL_FILTER = level };
        }
    } else {
        use std::sync::Mutex;

        lazy_static::lazy_static! {
            static ref LEVEL_FILTER : Mutex<LevelFilter> = Mutex::new(LevelFilter::Trace);
        }
        #[inline(always)]
        pub fn log_level_enabled(level: Level) -> bool {
            *LEVEL_FILTER.lock().unwrap() >= level
        }
        pub fn set_log_level(level: LevelFilter) {
            *LEVEL_FILTER.lock().unwrap() = level;
        }

        #[cfg(feature = "external-logger")]
        mod workflow_logger {
            use log::{ Level, LevelFilter, Record, Metadata, SetLoggerError };

            pub struct WorkflowLogger;

            impl log::Log for WorkflowLogger {
                fn enabled(&self, metadata: &Metadata) -> bool {
                    super::log_level_enabled(metadata.level())
                }
    
                fn log(&self, record: &Record) {
                    if self.enabled(record.metadata()) {
                        match record.metadata().level() {
                            Level::Error => { super::error_impl(record.args()); },
                            Level::Warn => { super::warn_impl(record.args()); },
                            Level::Info => { super::info_impl(record.args()); },
                            Level::Debug => { super::debug_impl(record.args()); },
                            Level::Trace => { super::trace_impl(record.args()); },
                        }
                    }
                }
    
                fn flush(&self) {}
            }

            static LOGGER: WorkflowLogger = WorkflowLogger;

            pub fn init() -> Result<(), SetLoggerError> {
                log::set_logger(&LOGGER)
                    .map(|()| log::set_max_level(LevelFilter::Trace))
            }
        }

        #[cfg(feature = "external-logger")]
        pub fn init() -> Result<(), log::SetLoggerError> {
            workflow_logger::init()
        }

    }
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    // use super::*;
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);
        #[wasm_bindgen(js_namespace = console)]
        pub fn warn(s: &str);
        #[wasm_bindgen(js_namespace = console)]
        pub fn error(s: &str);
    }
}

pub mod impls {
    use super::*;

    #[inline(always)]
    pub fn error_impl(args : &fmt::Arguments<'_>) {
        if log_level_enabled(Level::Error) {
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    workflow_log::wasm::error(&args.to_string());
                } else if #[cfg(target_arch = "bpf")] {
                    solana_program::log::sol_log(&args.to_string());
                } else {
                    println!("{}",args.to_string());
                }
            }
        }
    }

    #[inline(always)]
    pub fn warn_impl(args : &fmt::Arguments<'_>) {
        if log_level_enabled(Level::Warn) {
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    workflow_log::wasm::warn(&args.to_string());
                } else if #[cfg(target_arch = "bpf")] {
                    solana_program::log::sol_log(&args.to_string());
                } else {
                    println!("{}",args.to_string());
                }
            }
        }
    }

    #[inline(always)]
    pub fn info_impl(args : &fmt::Arguments<'_>) {
        if log_level_enabled(Level::Info) {
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    workflow_log::wasm::log(&args.to_string());
                } else if #[cfg(target_arch = "bpf")] {
                    solana_program::log::sol_log(&args.to_string());
                } else {
                    println!("{}",args.to_string());
                }
            }
        }
    }

    #[inline(always)]
    pub fn debug_impl(args : &fmt::Arguments<'_>) {
        if log_level_enabled(Level::Debug) {
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    workflow_log::wasm::log(&args.to_string());
                } else if #[cfg(target_arch = "bpf")] {
                    solana_program::log::sol_log(&args.to_string());
                } else {
                    println!("{}",args.to_string());
                }
            }
        }
    }

    #[inline(always)]
    pub fn trace_impl(args : &fmt::Arguments<'_>) {
        if log_level_enabled(Level::Trace) {
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    workflow_log::wasm::log(&args.to_string());
                } else if #[cfg(target_arch = "bpf")] {
                    solana_program::log::sol_log(&args.to_string());
                } else {
                    println!("{}",args.to_string());
                }
            }
        }
    }
}

#[macro_export]
macro_rules! log_error {
    ($($t:tt)*) => (
        workflow_log::impls::error_impl(&format_args!($($t)*))
    )
}

#[macro_export]
macro_rules! log_warning {
    ($($t:tt)*) => (
        workflow_log::impls::warn_impl(&format_args!($($t)*))
    )
}

#[macro_export]
macro_rules! log_info {
    ($($t:tt)*) => (
        workflow_log::impls::info_impl(&format_args!($($t)*))
    )
}

#[macro_export]
macro_rules! log_debug {
    ($($t:tt)*) => (
        workflow_log::impls::debug_impl(&format_args!($($t)*))
    )
}

#[macro_export]
macro_rules! log_trace {
    ($($t:tt)*) => (
        workflow_log::impls::trace_impl(&format_args!($($t)*))
    )
}

pub use log_error;
pub use log_warning;
pub use log_info;
pub use log_debug;
pub use log_trace;

#[cfg(not(target_arch = "bpf"))]
pub fn trace_hex(data : &[u8]) {
    let hex = format_hex(data);
    log_trace!("{}", hex);

}

#[cfg(not(target_arch = "bpf"))]
pub fn format_hex(data : &[u8]) -> String {
    let view = hexplay::HexViewBuilder::new(data)
    .address_offset(0)
    .row_width(16)
    .finish();

    format!("{}",view).into()
}