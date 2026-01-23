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

## Testing with Mocks

The `mock` feature provides a `MockHttpClient` for testing:

```rs
use anyhttp::mock::{MockHttpClient, MockResponse};
use anyhttp::HttpClient;
use http::{Request, StatusCode};

#[tokio::test]
async fn test_api_call() {
    let client = MockHttpClient::new()
        .with_response(MockResponse::new(StatusCode::OK).body(b"hello world"));

    let request = Request::get("https://example.com").body(vec![]).unwrap();
    let response = client.execute(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.bytes().await.unwrap().as_ref(), b"hello world");
}
```

You can also queue multiple responses and simulate errors:

```rs
let client = MockHttpClient::new()
    .with_response(MockResponse::new(StatusCode::OK).body(b"first"))
    .with_error("connection failed")
    .with_response(MockResponse::new(StatusCode::CREATED).body(b"third"));

// Responses are returned in FIFO order
```
