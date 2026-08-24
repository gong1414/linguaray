use reqwest::{Client, RequestBuilder, Response};

/// Small HTTP transport shared by traditional provider adapters.
#[derive(Debug, Clone)]
pub struct HttpClient {
    endpoint: String,
    inner: Client,
}

#[allow(dead_code)]
impl HttpClient {
    pub fn new(endpoint: impl Into<String>, inner: Client) -> Self {
        Self {
            endpoint: endpoint.into().trim_end_matches('/').to_owned(),
            inner,
        }
    }

    pub fn client(&self) -> &Client {
        &self.inner
    }

    pub fn base_url(&self) -> &str {
        &self.endpoint
    }

    pub fn join_url(&self, path: &str) -> String {
        let suffix = if path.starts_with('/') {
            path.to_owned()
        } else {
            format!("/{path}")
        };
        format!("{}{suffix}", self.endpoint)
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.inner.post(self.join_url(path))
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.inner.get(self.join_url(path))
    }

    pub async fn execute(&self, request: RequestBuilder) -> reqwest::Result<Response> {
        request.send().await
    }
}
