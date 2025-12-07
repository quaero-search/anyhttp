use http::Request;

mod response;
pub use response::{HttpResponse, Response};

#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    /// Sends the specified request using the client.
    async fn execute(
        &self,
        request: Request<Vec<u8>>,
    ) -> std::result::Result<Response, anyhow::Error>;
}
