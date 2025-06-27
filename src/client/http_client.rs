use anyhow::Result;
use reqwest::{Client, Method, Response, Url};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// HTTP Client wrapper with Tower middleware support
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    base_url: Option<String>,
    default_timeout: Duration,
}

/// Builder for creating HTTP clients with various configurations
pub struct HttpClientBuilder {
    timeout: Duration,
    base_url: Option<String>,
    user_agent: Option<String>,
    default_headers: reqwest::header::HeaderMap,
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            base_url: None,
            user_agent: Some(format!("reprime-backend/{}", env!("CARGO_PKG_VERSION"))),
            default_headers: reqwest::header::HeaderMap::new(),
        }
    }
}

impl HttpClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn base_url<S: Into<String>>(mut self, base_url: S) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn default_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: reqwest::header::IntoHeaderName,
        V: AsRef<str>,
    {
        if let Ok(header_value) = reqwest::header::HeaderValue::from_str(value.as_ref()) {
            self.default_headers.insert(key, header_value);
        }
        self
    }

    pub fn build(self) -> Result<HttpClient> {
        let mut client_builder = Client::builder()
            .timeout(self.timeout)
            .default_headers(self.default_headers);

        if let Some(user_agent) = self.user_agent {
            client_builder = client_builder.user_agent(user_agent);
        }

        let client = client_builder.build()?;

        Ok(HttpClient {
            client,
            base_url: self.base_url,
            default_timeout: self.timeout,
        })
    }
}

impl HttpClient {
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::new()
    }

    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// Create a client with base URL
    pub fn with_base_url<S: Into<String>>(base_url: S) -> Result<Self> {
        Self::builder().base_url(base_url).build()
    }

    /// Resolve URL with base URL if set
    fn resolve_url(&self, url: &str) -> Result<Url> {
        match &self.base_url {
            Some(base) => {
                let base_url = Url::parse(base)?;
                Ok(base_url.join(url)?)
            }
            None => Ok(Url::parse(url)?),
        }
    }

    /// GET request
    pub async fn get<T>(&self, url: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = self.resolve_url(url)?;
        let response = self.client.get(url).send().await?;
        self.handle_response(response).await
    }

    /// POST request with JSON body
    pub async fn post<B, T>(&self, url: &str, body: &B) -> Result<T>
    where
        B: Serialize,
        T: for<'de> Deserialize<'de>,
    {
        let url = self.resolve_url(url)?;
        let response = self.client.post(url).json(body).send().await?;
        self.handle_response(response).await
    }

    /// PUT request with JSON body
    pub async fn put<B, T>(&self, url: &str, body: &B) -> Result<T>
    where
        B: Serialize,
        T: for<'de> Deserialize<'de>,
    {
        let url = self.resolve_url(url)?;
        let response = self.client.put(url).json(body).send().await?;
        self.handle_response(response).await
    }

    /// DELETE request
    pub async fn delete<T>(&self, url: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = self.resolve_url(url)?;
        let response = self.client.delete(url).send().await?;
        self.handle_response(response).await
    }

    /// Generic request method
    pub async fn request<B, T>(&self, method: Method, url: &str, body: Option<&B>) -> Result<T>
    where
        B: Serialize,
        T: for<'de> Deserialize<'de>,
    {
        let url = self.resolve_url(url)?;
        let mut request = self.client.request(method, url);

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Handle response and deserialize JSON
    async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();
        
        if status.is_success() {
            let json = response.json::<T>().await?;
            Ok(json)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("HTTP {} error: {}", status, error_text))
        }
    }

    /// Get raw response for custom handling
    pub async fn get_response(&self, url: &str) -> Result<Response> {
        let url = self.resolve_url(url)?;
        let response = self.client.get(url).send().await?;
        Ok(response)
    }

    /// Health check method
    pub async fn health_check(&self, url: &str) -> Result<bool> {
        match self.get_response(url).await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}
