use std::fmt;

use bytes::Bytes;
use http::StatusCode;
use url::Url;

/// An HTTP response wrapper that abstracts over different HTTP client implementations.
pub struct Response {
    inner: Box<dyn HttpResponse>,
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("url", &self.url().as_str())
            .field("status", &self.status())
            .finish()
    }
}

impl Response {
    /// Creates a new response from any type implementing `HttpResponse`.
    pub fn new(inner: impl HttpResponse + 'static) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }

    /// Consumes the response and returns the body as bytes.
    pub async fn bytes(self) -> anyhow::Result<Bytes> {
        self.inner.bytes().await
    }

    /// Consumes the response and returns the body as a stream of bytes.
    #[cfg(feature = "stream")]
    pub fn bytes_stream(
        self,
    ) -> std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<Bytes>> + Send>> {
        self.inner.bytes_stream()
    }

    /// Returns the URL of the response.
    pub fn url(&self) -> &Url {
        self.inner.url()
    }

    /// Returns the HTTP status code of the response.
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }
}

/// A trait for HTTP response implementations.
#[async_trait::async_trait]
pub trait HttpResponse: Send + Sync {
    /// Consumes the response and returns the body as bytes.
    async fn bytes(self: Box<Self>) -> anyhow::Result<Bytes>;

    /// Consumes the response and returns the body as a stream of bytes.
    #[cfg(feature = "stream")]
    fn bytes_stream(
        self: Box<Self>,
    ) -> std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<Bytes>> + Send>>;

    /// Returns the URL of the response.
    fn url(&self) -> &Url;

    /// Returns the HTTP status code of the response.
    fn status(&self) -> StatusCode;
}
