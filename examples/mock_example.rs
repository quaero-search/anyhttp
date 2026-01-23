use anyhttp::HttpClient;
use anyhttp::mock::{MockHttpClient, MockResponse};
use http::{Request, StatusCode};

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
    let client = MockHttpClient::new()
        .with_response(MockResponse::new(StatusCode::OK).body(b"{\"products\": []}"));

    let foo = Foo::new(client);

    let data = foo.fetch_products().await;
    println!("{:#?}", data)
}
