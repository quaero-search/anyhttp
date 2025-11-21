use std::ops::{Deref, DerefMut};
use url::Url;

#[derive(Clone)]
pub struct Response<T> {
    url: Box<Url>,
    inner: http::Response<T>,
}

impl<T> Response<T> {
    pub fn new(inner: http::Response<T>, url: Url) -> Self {
        Self {
            url: Box::new(url),
            inner,
        }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn into_body(self) -> T {
        self.inner.into_body()
    }
}

impl<T> Deref for Response<T> {
    type Target = http::Response<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Response<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
