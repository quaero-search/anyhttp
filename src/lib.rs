#[allow(hidden_glob_reexports)]
mod response;
pub use response::Response;

mod adapters;
pub use adapters::*;

pub use http::{Response as RawResponse, *};

#[async_trait::async_trait]
pub trait AnyHttpClient {
    /// The underlying Error type for the client.
    type Error: std::error::Error + Send + Sync;

    /// Executes the specified request using the client.
    async fn execute(
        &self,
        request: Request<Vec<u8>>,
    ) -> std::result::Result<Response<Vec<u8>>, Self::Error>;
}
