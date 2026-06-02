//! NeuDel-II Studio — pure-Rust GPUI editor.
//!
//! No HTML, no JS, no bundler. The whole UI is a Rust binary that talks to
//! the engine crate directly.

mod model;
mod services;
mod state;
mod ui;

use gpui::Application;

fn main() {
    Application::new().run(|cx| ui::root::run(cx));
}
