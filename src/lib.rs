//! A minimal, trait-based HTTP client abstraction.
//!
//! This crate provides traits for abstracting over HTTP clients, allowing you to write
//! code that is generic over the HTTP implementation used.
//!
//! # Features
//!
//! - `reqwest` - Implements `HttpClient` for `reqwest::Client`
//! - `mock` - Provides a mock HTTP client for testing
//! - `stream` - Enables streaming response bodies

#![warn(missing_docs)]

use http::Request;

mod response;
pub use response::{HttpResponse, Response};

/// Reqwest HTTP client implementation.
#[cfg(feature = "reqwest")]
pub mod reqwest;

/// Mock HTTP client for testing.
#[cfg(feature = "test-support")]
pub mod mock;

/// A trait for HTTP clients that can execute requests.
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    /// Sends the specified request and returns the response.
    async fn execute(
        &self,
        request: Request<Vec<u8>>,
    ) -> std::result::Result<Response, anyhow::Error>;
}
