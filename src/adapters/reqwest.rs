use crate::{AnyHttpClient, Request, response::Response};
use http_body_util::BodyExt;

#[async_trait::async_trait]
impl AnyHttpClient for reqwest::Client {
    type Error = reqwest::Error;

    async fn execute(&self, request: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Self::Error> {
        convert_response(self.execute(convert_request(request)?).await?).await
    }
}

#[inline]
fn convert_request(request: Request<Vec<u8>>) -> Result<reqwest::Request, reqwest::Error> {
    request.try_into()
}

#[inline]
async fn convert_response(res: reqwest::Response) -> Result<Response<Vec<u8>>, reqwest::Error> {
    let url = res.url().clone();
    let mut inner_res = http::Response::from(res);

    let body = inner_res.body_mut().collect().await?.to_bytes().to_vec();

    Ok(Response::new(inner_res.map(move |_| body), url))
}
