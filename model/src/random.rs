mod shuffled;
mod smartish;

/// Generate a completely random board. Chances are, it's unsolvable.
pub use shuffled::shuffled_random;
/// Generate a board that's (probably) solvable by (mostly) "un-playing" a solution.
pub use smartish::smartish_random;
