pub mod types;
pub mod inst;
pub mod block;
pub mod func;
pub mod builder;

pub use types::Type;
pub use block::Block;
pub use func::Function;
pub use builder::{ILBuilder, OptId};