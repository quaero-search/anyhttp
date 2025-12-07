use bytes::Bytes;
use url::Url;

#[derive(Clone)]
pub struct Response<I: HttpResponse> {
    inner: I,
}

impl<I: HttpResponse> Response<I> {
    pub fn new(inner: I) -> Self {
        Self { inner }
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
    async fn bytes(self) -> anyhow::Result<Bytes>;

    #[cfg(feature = "stream")]
    fn bytes_stream(
        self,
    ) -> std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<Bytes>> + Send>>;

    fn url(&self) -> &Url;
}
