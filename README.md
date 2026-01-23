# anyhttp

This crate provides a way to make asynchronous http requests without locking consumers into using a specific http library.

## Example Usage

```rs
use anyhttp::HttpClient;
use http::Request;

struct Foo<C: HttpClient + 'static> {
    client: C,
}

impl<C: HttpClient + 'static> Foo<C> {
    fn new(client: C) -> Self {
        Self { client }
    }
}

impl<C: HttpClient + 'static> Foo<C> {
    async fn fetch_products(&self) -> anyhow::Result<String> {
        let request = Request::get("https://dummyjson.com/products").body(vec![])?;
        let response = self.client.execute(request).await?;

        let body_bytes = response.bytes().await?;

        Ok(String::from_utf8_lossy(&body_bytes).into_owned())
    }
}

#[tokio::main]
async fn main() {
    let foo = Foo::new(reqwest::Client::new());

    let data = foo.fetch_products().await;
    println!("{:#?}", data)
}
```

## Features

- `reqwest` - Implements `HttpClient` for `reqwest::Client`
- `mock` - Provides a mock HTTP client for testing
- `stream` - Enables streaming response bodies
