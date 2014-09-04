#![feature(phase, globs, unboxed_closures)]

#[phase(plugin)] extern crate regex_macros;
extern crate regex;

pub use client::{Client, ClientBuilder};
pub use message::Message;

pub mod client;
pub mod constants;
pub mod message;
