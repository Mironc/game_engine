//! This library purpose is to provide capabilities to draw effectively using vulkan renderer.
//!
//! It was designed to be used with winit paired with parallelazation for this render graph was implemented.
//!
//! It's hardly bond to vulkan and other graphics APIs are not going to be supported (atleast for now)
pub mod context;
pub mod device;
pub mod instance;
pub mod queue;
pub mod render_graph;
pub mod rendering;
pub mod swapchain;

extern crate self as graphics;
