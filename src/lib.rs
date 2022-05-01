mod errors;
mod index;
mod map;
mod shm;
#[cfg(test)]
mod tests;

pub use errors::ShmapError;
pub use map::Shmap;
