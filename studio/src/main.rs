//! NeuDel-II Studio — pure-Rust GPUI editor.
//!
//! No HTML, no JS, no bundler. The whole UI is a Rust binary that talks to
//! the engine crate directly.

mod model;
mod services;
mod state;
mod ui;

fn main() {
    gpui_platform::application().run(|cx| ui::root::run(cx));
}
