//! Reverse-engineered **web** Gemini image generation via cookie replay — no official API.
//!
//! Load your Google cookies, [`Client::connect`] to an account by its `/u/N` index,
//! then [`Client::generate_image`] and [`Client::download_bytes`]. The session re-mints
//! its own short-lived `1PSIDTS` on connect and can be persisted via [`cache`], so it
//! stays alive browser-free across runs. See the `examples/` directory for runnable usage.

mod auth;
pub mod cache;
mod client;
mod cookies;
mod download;
mod endpoints;
mod error;
mod generate;
mod parse;
mod rotate;

pub use client::Client;
pub use cookies::{load_netscape, Cookie};
pub use error::Error;
pub use parse::GeneratedImage;
