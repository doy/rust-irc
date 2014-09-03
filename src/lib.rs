#![feature(phase, globs)]

#[phase(plugin)] extern crate regex_macros;
extern crate regex;

mod constants;
mod message;
