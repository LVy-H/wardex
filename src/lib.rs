#![doc = include_str!("../README.md")]

pub mod config;
pub mod core;
pub mod engine;
pub mod output;
#[cfg(feature = "tui")]
pub mod tui;
pub mod utils;
