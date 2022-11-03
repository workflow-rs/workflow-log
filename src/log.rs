use cfg_if::cfg_if;
use std::fmt;

cfg_if! {
    if #[cfg(target_os = "solana")] {
        pub use workflow_log::levels::{ Level, LevelFilter };
    } else {
        use std::sync::Arc;
        pub use log::{ Level, LevelFilter };
        use downcast::{ downcast_sync, AnySync };
        pub use hexplay::{self, HexViewBuilder};
        pub use termcolor::Buffer;
        //use core::ops::Range;

        pub struct ColorHexView<'a>{
            pub builder: HexViewBuilder<'a>,
            pub color_start: usize
        }
        impl<'a> ColorHexView<'a>{
            pub fn new(builder:HexViewBuilder<'a>, colors:Vec<(&'a str, usize)>)->Self{
                Self{
                    builder,
                    color_start:0
                }.add_colors(colors)
            }

            pub fn add_colors(mut self, colors:Vec<(&'a str, usize)>)->Self{
                let mut builder = self.builder;
                for (color, len) in colors{
                    let end = self.color_start+len;
                    let range = self.color_start..end;
                    self.color_start = end;
                    builder = builder.add_color(color, range);
                }
                self.builder = builder;
                self
            }

            pub fn add_colors_with_range(mut self, colors:Vec<(&'a str, std::ops::Range<usize>)>)->Self{
                let mut builder = self.builder;
                for (color, range) in colors{
                    builder = builder.add_color(color, range);
                }
                self.builder = builder;
                self
            }

            pub fn try_print(self)->std::result::Result<(), String>{
                let mut buf = Buffer::ansi();
                match self.builder.finish().fmt(&mut buf){
                    Ok(()) => {
                        match String::from_utf8(buf.as_slice().to_vec()){
                            Ok(str)=>{
                                log_trace!("{}", str);
                            }
                            Err(_)=>{
                                return Err("Unable to convert HexView to string".to_string());
                            }
                        }
                    },
                    Err(_) => {
                        return Err("Unable to format HexView".to_string());
                    }
                }
                Ok(())
            }
        }

        pub trait Sink : AnySync {
            fn write(&self, level : Level, args : &fmt::Arguments<'_>) -> bool;
        }
        
        struct SinkHandler {
            // #[allow(dead_code)]
            sink : Arc<dyn Sink>, // + Send + Sync + 'static>,
        }

        downcast_sync!(dyn Sink);
    }
}

cfg_if! {
    if #[cfg(target_os = "solana")] {
        #[inline(always)]
        pub fn log_level_enabled(_level: Level) -> bool { 
            true
        }
    } else if #[cfg(target_arch = "wasm32")] {
        static mut LEVEL_FILTER : LevelFilter = LevelFilter::Trace;
        #[inline(always)]
        pub fn log_level_enabled(level: Level) -> bool { 
            unsafe { LEVEL_FILTER >= level } 
        }
        pub fn set_log_level(level: LevelFilter) { 
            unsafe { LEVEL_FILTER = level };
        }
        cfg_if! {
            if #[cfg(feature = "sink")] {
                static mut SINK : Option<SinkHandler> = None;
                // pub fn pipe(sink : Arc<dyn Sink + Send + Sync + 'static>) {
                pub fn pipe(sink : Option<Arc<dyn Sink>>) {
                    match sink {
                        Some(sink) => { unsafe { SINK = Some(SinkHandler { sink }); } },
                        None => { unsafe { SINK = None; } }
                    }
                }
                #[inline(always)]
                fn to_sink(level : Level, args : &fmt::Arguments<'_>) -> bool {
                    let sink = unsafe { &SINK };
                    match sink {
                        Some(handler) => {
                            handler.sink.write(level, args)
                        },
                        None => { false }
                    }
                }
            }
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
        cfg_if! {
            if #[cfg(feature = "sink")] {
                lazy_static::lazy_static! {
                    static ref SINK : Mutex<Option<SinkHandler>> = Mutex::new(None);
                }
                // pub fn pipe(sink : Option<Arc<dyn Sink + Send + Sync + 'static>>) {
                pub fn pipe(sink : Option<Arc<dyn Sink>>) {
                    match sink {
                        Some(sink) => { *SINK.lock().unwrap() = Some(SinkHandler { sink }); },
                        None => { *SINK.lock().unwrap() = None; }
                    }
                    
                }
                #[inline(always)]
                fn to_sink(level : Level, args : &fmt::Arguments<'_>) -> bool {
                    match SINK.lock().unwrap().as_ref() {
                        Some(handler) => {
                            handler.sink.write(level, args)
                        },
                        None => { false }
                    }
                }
            }
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

    pub fn error_impl(args : &fmt::Arguments<'_>) {
        if log_level_enabled(Level::Error) {
            #[cfg(all(not(target_os = "solana"),feature = "sink"))] {
                if to_sink(Level::Error, args) {
                    return;
                }
            }
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    workflow_log::wasm::error(&args.to_string());
                } else if #[cfg(target_os = "solana")] {
                    solana_program::log::sol_log(&args.to_string());
                } else {
                    println!("{}",args.to_string());
                }
            }
        }
    }

    pub fn warn_impl(args : &fmt::Arguments<'_>) {
        if log_level_enabled(Level::Warn) {
            #[cfg(all(not(target_os = "solana"),feature = "sink"))] {
                if to_sink(Level::Warn, args) {
                    return;
                }
            }
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    workflow_log::wasm::warn(&args.to_string());
                } else if #[cfg(target_os = "solana")] {
                    solana_program::log::sol_log(&args.to_string());
                } else {
                    println!("{}",args.to_string());
                }
            }
        }
    }

    pub fn info_impl(args : &fmt::Arguments<'_>) {
        if log_level_enabled(Level::Info) {
            #[cfg(all(not(target_os = "solana"),feature = "sink"))] {
                if to_sink(Level::Info, args) {
                    return;
                }
            }
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    workflow_log::wasm::log(&args.to_string());
                } else if #[cfg(target_os = "solana")] {
                    solana_program::log::sol_log(&args.to_string());
                } else {
                    println!("{}",args.to_string());
                }
            }
        }
    }

    pub fn debug_impl(args : &fmt::Arguments<'_>) {
        if log_level_enabled(Level::Debug) {
            #[cfg(all(not(target_os = "solana"),feature = "sink"))] {
                if to_sink(Level::Debug, args) {
                    return;
                }
            }
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    workflow_log::wasm::log(&args.to_string());
                } else if #[cfg(target_os = "solana")] {
                    solana_program::log::sol_log(&args.to_string());
                } else {
                    println!("{}",args.to_string());
                }
            }
        }
    }

    pub fn trace_impl(args : &fmt::Arguments<'_>) {
        if log_level_enabled(Level::Trace) {
            #[cfg(all(not(target_os = "solana"),feature = "sink"))] {
                if to_sink(Level::Trace, args) {
                    return;
                }
            }
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    workflow_log::wasm::log(&args.to_string());
                } else if #[cfg(target_os = "solana")] {
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

#[cfg(not(target_os = "solana"))]
pub fn trace_hex(data : &[u8]) {
    let hex = format_hex(data);
    log_trace!("{}", hex);

}

#[cfg(not(target_os = "solana"))]
pub fn format_hex(data : &[u8]) -> String {
    let view = hexplay::HexViewBuilder::new(data)
    .address_offset(0)
    .row_width(16)
    .finish();

    format!("{}",view).into()
}

#[cfg(not(target_os = "solana"))]
pub fn format_hex_with_colors<'a>(data : &'a[u8], colors:Vec<(&'a str, usize)>) -> ColorHexView {
    let view_builder = hexplay::HexViewBuilder::new(data)
        .address_offset(0)
        .row_width(16);

    ColorHexView::new(view_builder, colors)
}
