#![no_std]
#![deny(unsafe_code)]
#![deny(warnings)]

#[macro_use]
extern crate alloc;

pub use embedded_graphics;

pub mod color;
pub mod console;
pub mod escape_parser;
pub mod graphic;
pub mod text_buffer;
pub mod text_buffer_cache;
