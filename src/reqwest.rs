use crate::{HttpClient, HttpResponse, Response};

use bytes::Bytes;
use http::{Request, StatusCode};
use url::Url;

#[async_trait::async_trait]
impl HttpClient for reqwest::Client {
    async fn execute(&self, request: Request<Vec<u8>>) -> Result<Response, anyhow::Error> {
        Ok(Response::new(
            reqwest::Client::execute(self, request.try_into()?).await?,
        ))
    }
}

#[async_trait::async_trait]
impl HttpResponse for reqwest::Response {
    async fn bytes(self: Box<Self>) -> anyhow::Result<Bytes> {
        reqwest::Response::bytes(*self)
            .await
            .map_err(anyhow::Error::new)
    }

    #[cfg(feature = "stream")]
    fn bytes_stream(
        self: Box<Self>,
    ) -> std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<Bytes>> + Send>> {
        use futures::StreamExt;

        Box::pin(reqwest::Response::bytes_stream(*self).map(|res| res.map_err(anyhow::Error::new)))
    }

    fn url(&self) -> &Url {
        reqwest::Response::url(self)
    }

    fn status(&self) -> StatusCode {
        reqwest::Response::status(self)
    }
}
