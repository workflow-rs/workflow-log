
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

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! log_impl {
    ($($t:tt)*) => (
        // #[allow(unused_unsafe)]
        workflow_log::wasm::log(&format_args!($($t)*).to_string())
    )
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]
#[macro_export]
macro_rules! log_impl {
    ($($t:tt)*) => ( println!("{}",&format_args!($($t)*).to_string()) )
}

#[cfg(target_arch = "bpf")]
#[macro_export]
macro_rules! log_impl {
    ($($t:tt)*) => ( solana_program::msg!("{}",&format_args!($($t)*).to_string()) )
}

// ~

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! warn_impl {
    ($($t:tt)*) => (
        #[allow(unused_unsafe)]
        unsafe { workflow_log::wasm::warn(&format_args!($($t)*).to_string()) } 
    )
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]
#[macro_export]
macro_rules! warn_impl {
    ($($t:tt)*) => ( println!("{}",&format_args!($($t)*).to_string()) )
}

#[cfg(target_arch = "bpf")]
#[macro_export]
macro_rules! warn_impl {
    ($($t:tt)*) => ( solana_program::msg!("{}",&format_args!($($t)*).to_string()) )
}

// ~

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! err_impl {
    ($($t:tt)*) => (
        #[allow(unused_unsafe)]
        unsafe { workflow_log::wasm::error(&format_args!($($t)*).to_string()) } 
    )
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]
#[macro_export]
macro_rules! err_impl {
    ($($t:tt)*) => ( println!("{}",&format_args!($($t)*).to_string()) )
}

#[cfg(target_arch = "bpf")]
#[macro_export]
macro_rules! err_impl {
    ($($t:tt)*) => ( solana_program::msg!("{}",&format_args!($($t)*).to_string()) )
}

// ~

#[macro_export]
macro_rules! trace {
    ($($t:tt)*) => (
        workflow_log::log_impl!("{}",&format_args!($($t)*).to_string())
    )
}

#[macro_export]
macro_rules! log_trace {
    ($($t:tt)*) => (
        workflow_log::log_impl!("{}",&format_args!($($t)*).to_string())
    )
}

#[macro_export]
macro_rules! log_warning {
    ($($t:tt)*) => (
        workflow_log::warn_impl!("{}",&format_args!($($t)*).to_string())
    )
}

#[macro_export]
macro_rules! log_error {
    ($($t:tt)*) => (
        workflow_log::err_impl!("{}",&format_args!($($t)*).to_string())
    )
}

pub use trace;
pub use log_warning;
pub use log_error;

#[cfg(not(target_arch = "bpf"))]
pub fn trace_hex(data : &[u8]) {
    let hex = format_hex(data);
    trace!("{}", hex);

}

#[cfg(not(target_arch = "bpf"))]
pub fn format_hex(data : &[u8]) -> String {
    let view = hexplay::HexViewBuilder::new(data)
    .address_offset(0)
    .row_width(16)
    .finish();

    format!("{}",view).into()
}