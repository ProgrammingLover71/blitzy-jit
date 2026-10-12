pub mod types;
pub mod loc;
pub mod inst;
pub mod block;
pub mod func;
pub mod builder;

pub use loc::{Location, LocationContext};
pub use types::Type;
pub use block::Block;
pub use func::Function;
pub use builder::{ILBuilder, OptId};