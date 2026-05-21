/// NeuDel-II Game Engine
///
/// Core systems will be implemented here as separate modules:
///   ecs/       — Entity Component System (hecs)
///   renderer/  — wgpu-based 2D/3D renderer
///   physics/   — rapier2d integration
///   audio/     — rodio audio system
///   scripting/ — Rust-native, Java (JNI), Python (PyO3) bridges

pub mod ecs;
pub mod renderer;
pub mod physics;
pub mod audio;
pub mod scripting;
