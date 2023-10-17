pub mod block;
pub mod burn;
mod call;
pub mod helper;
pub mod message;

pub use call::{call_function, call_getter};
