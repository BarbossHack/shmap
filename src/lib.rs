mod errors;
mod map;
mod shm;
#[cfg(test)]
mod tests;

pub use map::{get, remove, set};
