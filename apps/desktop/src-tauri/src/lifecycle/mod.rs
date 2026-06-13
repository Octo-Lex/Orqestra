//! Orqestra Development Lifecycle (v0.2.0)
//!
//! 13-stage development lifecycle model with event-sourced state.
//! All state changes are append-only events in `.Orqestra/lifecycle/events.jsonl`.

pub mod commands;
pub mod event_log;
pub mod types;
