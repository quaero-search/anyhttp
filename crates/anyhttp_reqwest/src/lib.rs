use std::ops::{Deref, DerefMut};

use anyhttp::{HttpClient, HttpResponse, Response};

use bytes::Bytes;
use http::Request;
use url::Url;

#[async_trait::async_trait]
impl HttpClient for ReqwestClientWrapper {
    type Response = ReqwestResponseWrapper;

    async fn execute(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Self::Response>, anyhow::Error> {
        Ok(Response::new(ReqwestResponseWrapper::new(
            self.inner.execute(convert_request(request)?).await?,
        )))
    }
}

#[async_trait::async_trait]
impl HttpResponse for ReqwestResponseWrapper {
    async fn bytes(self) -> anyhow::Result<Bytes> {
        reqwest::Response::bytes(self.inner)
            .await
            .map_err(anyhow::Error::new)
    }

    #[cfg(feature = "stream")]
    fn bytes_stream(self) -> Pin<Box<dyn Stream<Item = anyhow::Result<Bytes>> + Send>> {
        use futures::{Stream, StreamExt};
        use std::pin::Pin;

        Box::pin(
            reqwest::Response::bytes_stream(self.inner).map(|res| res.map_err(anyhow::Error::new)),
        )
    }

    fn url(&self) -> &Url {
        reqwest::Response::url(&self.inner)
    }
}

#[inline]
fn convert_request(request: Request<Vec<u8>>) -> Result<reqwest::Request, reqwest::Error> {
    request.try_into()
}

pub struct ReqwestClientWrapper {
    inner: reqwest::Client,
}

impl ReqwestClientWrapper {
    pub fn new(client: reqwest::Client) -> ReqwestClientWrapper {
        Self { inner: client }
    }
}

impl Into<ReqwestClientWrapper> for reqwest::Client {
    fn into(self) -> ReqwestClientWrapper {
        ReqwestClientWrapper::new(self)
    }
}

impl Deref for ReqwestClientWrapper {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ReqwestClientWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct ReqwestResponseWrapper {
    inner: reqwest::Response,
}

impl ReqwestResponseWrapper {
    pub fn new(response: reqwest::Response) -> ReqwestResponseWrapper {
        Self { inner: response }
    }
}

impl Into<ReqwestResponseWrapper> for reqwest::Response {
    fn into(self) -> ReqwestResponseWrapper {
        ReqwestResponseWrapper::new(self)
    }
}

impl Deref for ReqwestResponseWrapper {
    type Target = reqwest::Response;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ReqwestResponseWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
