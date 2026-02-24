//! Rust port of `pixivpy3` (AppPixivAPI).
//!
//! Public API is kept similar to the Python library:
//! - `AppPixivAPI` is the main entry point.
//! - `PixivError` is the error type.

#![deny(clippy::unwrap_used)]

pub mod aapi;
pub mod error;
mod log;
pub mod models;
pub mod params;
pub mod token_manager;

pub use crate::aapi::AppPixivAPI;
pub use crate::error::PixivError;
pub(crate) use crate::log::*;
pub use crate::token_manager::TokenManager;
