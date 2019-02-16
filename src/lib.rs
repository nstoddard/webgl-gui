//! A work-in-progress GUI library for WebAssembly, using WebGL 2.
//!
//! This library currently also contains asset loading and a main loop, but these might
//! be moved to separate crates at some point.

#![deny(bare_trait_objects)]

mod assets;
mod color;
mod draw_2d;
mod event;
pub mod gui;
mod main_loop;
mod shader_header;
mod text;
pub mod widgets;

pub use crate::assets::*;
pub use crate::color::*;
pub use crate::draw_2d::Draw2d;
pub use crate::event::*;
pub use crate::gui::*;
pub use crate::main_loop::*;
pub use crate::shader_header::*;
pub use crate::text::Font;
