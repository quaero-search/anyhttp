use anyhttp::{AnyHttpClient, Request};

struct Foo<C: AnyHttpClient + 'static> {
    client: C,
}

impl<C: AnyHttpClient + 'static> Foo<C> {
    async fn fetch_products(&self) -> anyhow::Result<String> {
        let req: Request<Vec<_>> = Request::get("https://dummyjson.com/products").body(vec![])?;
        let res = self.client.execute(req).await?;

        Ok(String::from_utf8_lossy(res.body()).to_string())
    }
}

#[tokio::main]
async fn main() {
    let foo = Foo {
        client: reqwest::Client::new(),
    };

    let data = foo.fetch_products().await;
    println!("{:#?}", data)
}
