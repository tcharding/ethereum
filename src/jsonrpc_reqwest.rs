//! Asynchronous JSON RPC client using `reqwest`.
use std::fmt::Debug;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
pub use url::Url;

#[derive(Clone, Debug)]
pub struct Client {
    inner: reqwest::Client,
    url: Url,
}

impl Client {
    pub fn new(base_url: Url) -> Self {
        Self {
            inner: reqwest::Client::new(),
            url: base_url,
        }
    }

    pub async fn send<Req, Res>(&self, request: Request<Req>) -> Result<Res>
    where
        Req: Debug + Serialize,
        Res: Debug + DeserializeOwned,
    {
        self.send_with_path("".into(), request).await
    }

    pub async fn send_with_path<Req, Res>(&self, path: String, request: Request<Req>) -> Result<Res>
    where
        Req: Debug + Serialize,
        Res: Debug + DeserializeOwned,
    {
        let url = self.url.clone().join(&path)?;

        let response = self
            .inner
            .post(url.clone())
            .json(&request)
            .send()
            .await
            .context("failed to send request")?
            .json::<Response<Res>>()
            .await
            .context("failed to deserialize JSON response as JSON-RPC response")?
            .payload
            .into_result()
            .with_context(|| {
                format!(
                    "JSON-RPC request {} failed",
                    serde_json::to_string(&request).expect("can always serialize to JSON")
                )
            })?;

        Ok(response)
    }
}

pub const JSONRPC_VERSION_2: &str = "2.0";

#[derive(serde::Serialize, Debug, Clone)]
pub struct Request<T> {
    id: String,
    jsonrpc: String,
    method: String,
    params: T,
}

impl<T> Request<T> {
    /// Construct a new request.
    pub fn new(method: &str, params: T, jsonrpc: String) -> Self {
        Self {
            id: "1".to_owned(),
            jsonrpc,
            method: method.to_owned(),
            params,
        }
    }

    /// Construct a new request using JSON RPC version 2.
    pub fn v2(method: &str, params: T) -> Self {
        Self::new(method, params, JSONRPC_VERSION_2.into())
    }
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct Response<R> {
    #[serde(flatten)]
    pub payload: ResponsePayload<R>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponsePayload<R> {
    Result(R),
    Error(JsonRpcError),
}

impl<R> ResponsePayload<R> {
    fn into_result(self) -> Result<R, JsonRpcError> {
        match self {
            ResponsePayload::Result(result) => Ok(result),
            ResponsePayload::Error(e) => Err(e),
        }
    }
}

#[derive(Debug, Deserialize, Error, PartialEq)]
#[error("JSON-RPC request failed with code {code}: {message}")]
pub struct JsonRpcError {
    code: i64,
    message: String,
}

pub fn serialize<T>(t: T) -> Result<serde_json::Value>
where
    T: Serialize,
{
    let value = serde_json::to_value(t).context("failed to serialize parameter")?;

    Ok(value)
}
