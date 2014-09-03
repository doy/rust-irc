#![feature(phase, globs)]

#[phase(plugin)] extern crate regex_macros;
extern crate regex;

pub use client::Client;

pub mod client;
mod constants;
mod message;
