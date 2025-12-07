use bytes::Bytes;
use url::Url;

pub struct Response {
    inner: Box<dyn HttpResponse>,
}

impl Response {
    pub fn new(inner: impl HttpResponse + 'static) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }

    pub async fn bytes(self) -> anyhow::Result<Bytes> {
        self.inner.bytes().await
    }

    #[cfg(feature = "stream")]
    pub fn bytes_stream(
        self,
    ) -> std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<Bytes>> + Send>> {
        self.inner.bytes_stream()
    }

    pub fn url(&self) -> &Url {
        self.inner.url()
    }
}

#[async_trait::async_trait]
pub trait HttpResponse {
    async fn bytes(self: Box<Self>) -> anyhow::Result<Bytes>;

    #[cfg(feature = "stream")]
    fn bytes_stream(
        self: Box<Self>,
    ) -> std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<Bytes>> + Send>>;

    fn url(&self) -> &Url;
}
