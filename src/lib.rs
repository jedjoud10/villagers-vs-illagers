#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
use wasm4::*;

#[no_mangle]
fn start() {
}

#[no_mangle]
fn update() {
}