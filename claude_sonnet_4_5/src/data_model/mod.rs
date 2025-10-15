// Data model module
// Each struct is defined in its own submodule

pub mod pips;
pub mod piece;
pub mod point;
pub mod board;
pub mod assignment;
pub mod constraint;
pub mod direction;
pub mod placement;
pub mod game;

// Re-export main types for convenience
pub use pips::Pips;
pub use piece::Piece;
pub use point::Point;
pub use board::Board;
pub use constraint::Constraint;
pub use direction::Direction;
pub use placement::Placement;
pub use game::Game;
