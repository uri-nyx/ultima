// organum.rs
/* Organum is a very simple library that provides basic abstractions
and general interfaces to model and tailor pseudo-hardware modules 
for simple emulators. The code is mostly based on this article: 
https://dev.to/transistorfet/making-a-68000-emulator-in-rust-1kfk */

pub mod core;
pub mod debugger;
pub mod error;
pub mod interrupts;
pub mod premade;
pub mod server;
pub mod sys;