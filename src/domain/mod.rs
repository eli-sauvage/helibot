//! Rules that decide who earns, what rank they hold and how the board reads.
//!
//! Everything here is pure: no Discord, no database. That is what makes it testable,
//! and these are the parts that were worth preserving from the previous version.

pub mod leaderboard;
pub mod tiers;
pub mod voice;
