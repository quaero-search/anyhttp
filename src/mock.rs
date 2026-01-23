use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use http::{Request, StatusCode};
use url::Url;

use crate::{HttpClient, HttpResponse, Response};

/// A mock HTTP client for testing purposes.
///
/// # Example
///
/// ```
/// use anyhttp::mock::{MockHttpClient, MockResponse};
/// use anyhttp::HttpClient;
/// use http::{Request, StatusCode};
///
/// #[tokio::test]
/// async fn test_api_call() {
///     let client = MockHttpClient::new()
///         .with_response(MockResponse::new(StatusCode::OK).body(b"hello world"));
///
///     let request = Request::get("https://example.com").body(vec![]).unwrap();
///     let response = client.execute(request).await.unwrap();
///
///     assert_eq!(response.status(), StatusCode::OK);
///     assert_eq!(response.bytes().await.unwrap().as_ref(), b"hello world");
/// }
/// ```
#[derive(Clone)]
pub struct MockHttpClient {
    responses: Arc<Mutex<VecDeque<MockResponseOrError>>>,
    requests: Arc<Mutex<Vec<Request<Vec<u8>>>>>,
}

enum MockResponseOrError {
    Response(MockResponse),
    Error(String),
}

impl MockHttpClient {
    /// Creates a new mock HTTP client with no pre-configured responses.
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::new())),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Adds a response to be returned by the next call to `execute`.
    /// Responses are returned in FIFO order.
    pub fn with_response(self, response: MockResponse) -> Self {
        self.responses
            .lock()
            .unwrap()
            .push_back(MockResponseOrError::Response(response));
        self
    }

    /// Adds an error to be returned by the next call to `execute`.
    /// Errors are returned in FIFO order along with responses.
    pub fn with_error(self, error: impl Into<String>) -> Self {
        self.responses
            .lock()
            .unwrap()
            .push_back(MockResponseOrError::Error(error.into()));
        self
    }

    /// Queues a response to be returned by a future call to `execute`.
    pub fn queue_response(&self, response: MockResponse) {
        self.responses
            .lock()
            .unwrap()
            .push_back(MockResponseOrError::Response(response));
    }

    /// Queues an error to be returned by a future call to `execute`.
    pub fn queue_error(&self, error: impl Into<String>) {
        self.responses
            .lock()
            .unwrap()
            .push_back(MockResponseOrError::Error(error.into()));
    }

    /// Returns all requests that have been made to this client.
    pub fn requests(&self) -> Vec<Request<Vec<u8>>> {
        self.requests.lock().unwrap().clone()
    }

    /// Returns the number of requests that have been made to this client.
    pub fn request_count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }

    /// Clears all recorded requests.
    pub fn clear_requests(&self) {
        self.requests.lock().unwrap().clear();
    }

    /// Returns the last request made to this client, if any.
    pub fn last_request(&self) -> Option<Request<Vec<u8>>> {
        self.requests.lock().unwrap().last().cloned()
    }
}

impl Default for MockHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl HttpClient for MockHttpClient {
    async fn execute(&self, request: Request<Vec<u8>>) -> Result<Response, anyhow::Error> {
        self.requests.lock().unwrap().push(request.clone());

        let response_or_error = self
            .responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| {
                MockResponseOrError::Response(
                    MockResponse::new(StatusCode::OK).url(request.uri().to_string()),
                )
            });

        match response_or_error {
            MockResponseOrError::Response(mut response) => {
                if response.url.is_none() {
                    response.url = Some(
                        Url::parse(&request.uri().to_string())
                            .unwrap_or_else(|_| Url::parse("http://mock.test/").unwrap()),
                    );
                }
                Ok(Response::new(response))
            }
            MockResponseOrError::Error(error) => Err(anyhow::anyhow!("{}", error)),
        }
    }
}

/// A mock HTTP response for use with `MockHttpClient`.
pub struct MockResponse {
    status: StatusCode,
    body: Bytes,
    url: Option<Url>,
}

impl MockResponse {
    /// Creates a new mock response with the given status code.
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            body: Bytes::new(),
            url: None,
        }
    }

    /// Sets the response body.
    pub fn body(mut self, body: impl Into<Bytes>) -> Self {
        self.body = body.into();
        self
    }

    /// Sets the response URL.
    pub fn url(mut self, url: impl AsRef<str>) -> Self {
        self.url = Url::parse(url.as_ref()).ok();
        self
    }
}

#[async_trait::async_trait]
impl HttpResponse for MockResponse {
    async fn bytes(self: Box<Self>) -> anyhow::Result<Bytes> {
        Ok(self.body)
    }

    #[cfg(feature = "stream")]
    fn bytes_stream(
        self: Box<Self>,
    ) -> std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<Bytes>> + Send>> {
        Box::pin(futures::stream::once(async move { Ok(self.body) }))
    }

    fn url(&self) -> &Url {
        self.url
            .as_ref()
            .expect("MockResponse URL not set and no request was made")
    }

    fn status(&self) -> StatusCode {
        self.status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_client_returns_configured_response() {
        let client = MockHttpClient::new()
            .with_response(MockResponse::new(StatusCode::OK).body(b"hello world".as_slice()));

        let request = Request::get("https://example.com").body(vec![]).unwrap();
        let response = client.execute(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.bytes().await.unwrap().as_ref(), b"hello world");
    }

    #[tokio::test]
    async fn test_mock_client_returns_default_response_when_none_configured() {
        let client = MockHttpClient::new();

        let request = Request::get("https://example.com").body(vec![]).unwrap();
        let response = client.execute(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_mock_client_returns_responses_in_fifo_order() {
        let client = MockHttpClient::new()
            .with_response(MockResponse::new(StatusCode::OK).body(b"first".as_slice()))
            .with_response(MockResponse::new(StatusCode::CREATED).body(b"second".as_slice()))
            .with_response(MockResponse::new(StatusCode::ACCEPTED).body(b"third".as_slice()));

        let request = Request::get("https://example.com").body(vec![]).unwrap();

        let response1 = client.execute(request.clone()).await.unwrap();
        assert_eq!(response1.status(), StatusCode::OK);
        assert_eq!(response1.bytes().await.unwrap().as_ref(), b"first");

        let response2 = client.execute(request.clone()).await.unwrap();
        assert_eq!(response2.status(), StatusCode::CREATED);
        assert_eq!(response2.bytes().await.unwrap().as_ref(), b"second");

        let response3 = client.execute(request).await.unwrap();
        assert_eq!(response3.status(), StatusCode::ACCEPTED);
        assert_eq!(response3.bytes().await.unwrap().as_ref(), b"third");
    }

    #[tokio::test]
    async fn test_mock_client_returns_error() {
        let client = MockHttpClient::new().with_error("connection failed");

        let request = Request::get("https://example.com").body(vec![]).unwrap();
        let result = client.execute(request).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "connection failed");
    }

    #[tokio::test]
    async fn test_mock_client_records_requests() {
        let client = MockHttpClient::new();

        let request1 = Request::get("https://example.com/first")
            .body(vec![])
            .unwrap();
        let request2 = Request::post("https://example.com/second")
            .body(b"body".to_vec())
            .unwrap();

        client.execute(request1).await.unwrap();
        client.execute(request2).await.unwrap();

        assert_eq!(client.request_count(), 2);

        let requests = client.requests();
        assert_eq!(requests[0].uri(), "https://example.com/first");
        assert_eq!(requests[1].uri(), "https://example.com/second");
        assert_eq!(requests[1].body(), b"body");
    }

    #[tokio::test]
    async fn test_mock_client_last_request() {
        let client = MockHttpClient::new();

        assert!(client.last_request().is_none());

        let request = Request::get("https://example.com").body(vec![]).unwrap();
        client.execute(request).await.unwrap();

        let last = client.last_request().unwrap();
        assert_eq!(last.uri(), "https://example.com");
    }

    #[tokio::test]
    async fn test_mock_client_clear_requests() {
        let client = MockHttpClient::new();

        let request = Request::get("https://example.com").body(vec![]).unwrap();
        client.execute(request).await.unwrap();

        assert_eq!(client.request_count(), 1);

        client.clear_requests();

        assert_eq!(client.request_count(), 0);
    }

    #[tokio::test]
    async fn test_mock_client_queue_response() {
        let client = MockHttpClient::new();

        client.queue_response(MockResponse::new(StatusCode::NOT_FOUND));

        let request = Request::get("https://example.com").body(vec![]).unwrap();
        let response = client.execute(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_mock_client_queue_error() {
        let client = MockHttpClient::new();

        client.queue_error("server error");

        let request = Request::get("https://example.com").body(vec![]).unwrap();
        let result = client.execute(request).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_response_with_custom_url() {
        let client = MockHttpClient::new()
            .with_response(MockResponse::new(StatusCode::OK).url("https://redirected.example.com"));

        let request = Request::get("https://example.com").body(vec![]).unwrap();
        let response = client.execute(request).await.unwrap();

        assert_eq!(response.url().as_str(), "https://redirected.example.com/");
    }

    #[tokio::test]
    async fn test_mock_response_uses_request_url_when_not_set() {
        let client = MockHttpClient::new()
            .with_response(MockResponse::new(StatusCode::OK).body(b"test".as_slice()));

        let request = Request::get("https://example.com/path")
            .body(vec![])
            .unwrap();
        let response = client.execute(request).await.unwrap();

        assert_eq!(response.url().as_str(), "https://example.com/path");
    }

    #[tokio::test]
    async fn test_mock_client_is_clone() {
        let client = MockHttpClient::new()
            .with_response(MockResponse::new(StatusCode::OK).body(b"response".as_slice()));

        let cloned = client.clone();

        let request = Request::get("https://example.com").body(vec![]).unwrap();
        let response = cloned.execute(request).await.unwrap();

        assert_eq!(response.bytes().await.unwrap().as_ref(), b"response");
        // Both clones share state
        assert_eq!(client.request_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_client_mixed_responses_and_errors() {
        let client = MockHttpClient::new()
            .with_response(MockResponse::new(StatusCode::OK))
            .with_error("temporary error")
            .with_response(MockResponse::new(StatusCode::CREATED));

        let request = Request::get("https://example.com").body(vec![]).unwrap();

        let response1 = client.execute(request.clone()).await;
        assert!(response1.is_ok());
        assert_eq!(response1.unwrap().status(), StatusCode::OK);

        let response2 = client.execute(request.clone()).await;
        assert!(response2.is_err());

        let response3 = client.execute(request).await;
        assert!(response3.is_ok());
        assert_eq!(response3.unwrap().status(), StatusCode::CREATED);
    }

    #[cfg(feature = "stream")]
    #[tokio::test]
    async fn test_mock_response_bytes_stream() {
        use futures::StreamExt;

        let client = MockHttpClient::new()
            .with_response(MockResponse::new(StatusCode::OK).body(b"streamed".as_slice()));

        let request = Request::get("https://example.com").body(vec![]).unwrap();
        let response = client.execute(request).await.unwrap();

        let mut stream = response.bytes_stream();
        let chunk = stream.next().await.unwrap().unwrap();

        assert_eq!(chunk.as_ref(), b"streamed");
    }
}
