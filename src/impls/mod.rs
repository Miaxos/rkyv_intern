#[cfg(feature = "alloc")]
mod alloc;

mod internment;
pub use internment::InternedRkyvString;
