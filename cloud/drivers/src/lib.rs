#![feature(core_intrinsics)]

mod camera;
mod devmem;
mod hex;
mod keys;
mod texture;
mod touchscreen;
mod vga;

const LW_BRIDGE_BASE: u64 = 0xFF200000;

pub use camera::{Camera, CameraGuard};
pub use devmem::DevMem;
pub use hex::HexDisplay;
pub use keys::{Keys, KeysPressed};
pub use texture::Texture;
pub use touchscreen::{TouchArea, TouchEvent, TouchScreen, TOUCHSCREEN_HEIGHT, TOUCHSCREEN_WIDTH};
pub use vga::VgaDisplay;
