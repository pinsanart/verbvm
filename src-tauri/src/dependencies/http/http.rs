use super::auth;
use super::error;

pub use auth::Auth;
pub use error::HttpError;

use reqwest::{Client, Method, RequestBuilder};
use serde::de::DeserializeOwned;
use std::time::Duration;

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("falha ao construir http client");

        Self { client }
    }

    fn request(&self, method: Method, url: &str, auth: Auth) -> RequestBuilder {
        let req = self.client.request(method, url);
        match auth {
            Auth::None => req,
            Auth::Bearer(token) => req.bearer_auth(token),
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str, auth: Auth) -> Result<T, HttpError> {
        let resp = self.request(Method::GET, url, auth).send().await?;
        Self::handle_response(resp).await
    }

    pub async fn post<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
        auth: Auth,
    ) -> Result<T, HttpError> {
        let resp = self
            .request(Method::POST, url, auth)
            .json(body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn put<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
        auth: Auth,
    ) -> Result<T, HttpError> {
        let resp = self
            .request(Method::PUT, url, auth)
            .json(body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn delete<T: DeserializeOwned>(&self, url: &str, auth: Auth) -> Result<T, HttpError> {
        let resp = self.request(Method::DELETE, url, auth).send().await?;
        Self::handle_response(resp).await
    }

    async fn handle_response<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T, HttpError> {
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(HttpError::Status { status, body });
        }
        Ok(resp.json::<T>().await?)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}