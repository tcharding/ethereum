//! JSON RPC client using `ureq` (blocking IO).
use std::fmt::Debug;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use ureq::{Agent, AgentBuilder};
pub use url::Url;

#[derive(Clone, Debug)]
pub struct Client {
    agent: ureq::Agent,
    url: Url,
}

impl Client {
    /// Construct a new client using `url` as the base URL to connect to.
    pub fn new(url: Url) -> Self {
        let agent: Agent = AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();

        Self { agent, url }
    }

    pub fn send<Req, Res>(&self, request: Request<Req>) -> Result<Res>
    where
        Req: Debug + Serialize,
        Res: Debug + DeserializeOwned,
    {
        self.send_with_path("".into(), request)
    }

    pub fn send_with_path<Req, Res>(&self, path: String, request: Request<Req>) -> Result<Res>
    where
        Req: Debug + Serialize,
        Res: Debug + DeserializeOwned,
    {
        let url = self.url.clone().join(&path)?;

        let response = self
            .agent
            .post(&url.to_string())
            .send_json(ureq::json!(&request))
            .context("failed to send request")?
            .into_json::<Response<Res>>()
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
