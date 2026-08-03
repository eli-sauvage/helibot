//! Helibot rewards time spent together in voice channels.
//!
//! The layers are kept apart on purpose: [`domain`] holds the rules and is pure, [`db`]
//! owns every query, and [`discord`] is the only place that talks to the gateway.

pub mod config;
pub mod db;
pub mod discord;
pub mod domain;
pub mod error;
pub mod state;
pub mod tasks;
