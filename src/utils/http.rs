use std::collections::HashMap;
use std::fmt;

use base64::encode;
use reqwest::{Client, Response, StatusCode};

use crate::utils::DuckResult;

pub trait HttpClient: Send + Sync {
    type Item: HttpResponse;
    fn send(&self, request: &HttpRequestBuilder) -> DuckResult<Self::Item>;
}

#[derive(Clone, PartialEq, Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
}

pub trait HttpResponse {
    fn status(&self) -> StatusCode;
    fn headers(&self) -> &reqwest::header::HeaderMap;
    fn body(&mut self) -> DuckResult<String>;
    fn deserialize_json<T: serde::de::DeserializeOwned>(&mut self) -> DuckResult<T>;
}

#[derive(Clone)]
pub struct HttpRequestBuilder {
    pub url: String,
    pub method: HttpMethod,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
}

impl HttpRequestBuilder {
    pub fn new(method: HttpMethod, url: String) -> Self {
        Self {
            url,
            method,
            body: None,
            headers: HashMap::new(),
        }
    }

    pub fn get<T: Into<String>>(url: T) -> Self {
        HttpRequestBuilder::new(HttpMethod::Get, url.into())
    }

    pub fn post(url: String) -> Self {
        HttpRequestBuilder::new(HttpMethod::Post, url)
    }

    pub fn put(url: String) -> Self {
        HttpRequestBuilder::new(HttpMethod::Put, url)
    }

    pub fn set_body(&mut self, body: String) {
        self.body = Some(body);
    }

    pub fn add_header<T: Into<String>>(&mut self, name: T, value: T) {
        self.headers.insert(name.into(), value.into());
    }

    // Borrowed from https://github.com/seanmonstar/reqwest/blob/master/src/blocking/request.rs#L234
    pub fn basic_auth<U, P>(&mut self, username: U, password: Option<P>)
    where
        U: fmt::Display,
        P: fmt::Display,
    {
        let auth = match password {
            Some(password) => format!("{}:{}", username, password),
            None => format!("{}:", username),
        };
        let header_value = format!("Basic {}", encode(&auth));
        self.add_header("Authorization", &*header_value);
    }
}

pub struct ReqwestClient {
    client: Client,
}

impl Default for ReqwestClient {
    fn default() -> Self {
        ReqwestClient::new()
    }
}

impl ReqwestClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl HttpResponse for Response {
    fn status(&self) -> StatusCode {
        self.status()
    }

    fn headers(&self) -> &reqwest::header::HeaderMap {
        self.headers()
    }

    fn deserialize_json<T: serde::de::DeserializeOwned>(&mut self) -> DuckResult<T> {
        let result: T = self.json()?;
        Ok(result)
    }
    fn body(&mut self) -> DuckResult<String> {
        let result = self.text()?;
        Ok(result)
    }
}

impl HttpClient for ReqwestClient {
    type Item = Response;

    fn send(&self, request: &HttpRequestBuilder) -> DuckResult<Response> {
        let mut builder = match &request.method {
            HttpMethod::Get => self.client.get(&request.url[..]),
            HttpMethod::Post => self.client.post(&request.url[..]),
            HttpMethod::Put => self.client.put(&request.url[..]),
        };

        // Copy headers
        for (name, value) in request.headers.iter() {
            builder = builder.header(name, value);
        }

        // Set the body
        if let Some(body) = &request.body {
            builder = builder.body(body.clone());
        };

        Ok(builder.send()?)
    }
}

#[cfg(test)]
pub struct MockHttpClient {
    pub responses: std::sync::Mutex<HashMap<String, MockHttpResponse>>,
    pub request: std::sync::Mutex<Vec<HttpRequestBuilder>>,
}

#[cfg(test)]
impl Default for MockHttpClient {
    fn default() -> Self {
        MockHttpClient::new()
    }
}

#[cfg(test)]
impl MockHttpClient {
    pub fn new() -> Self {
        MockHttpClient {
            responses: std::sync::Mutex::new(HashMap::new()),
            request: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn add_response(&self, builder: MockHttpResponseBuilder) {
        let mut responses = self.responses.lock().unwrap();
        let response = builder.build().unwrap();
        responses.insert(response.url.clone(), response);
    }

    pub fn get_sent_requests(&self) -> Vec<HttpRequestBuilder> {
        let requests = self.request.lock().unwrap();
        requests.clone()
    }
}

#[cfg(test)]
#[derive(Clone)]
pub struct MockHttpResponse {
    url: String,
    method: HttpMethod,
    status: StatusCode,
    body: Option<String>,
    headers: reqwest::header::HeaderMap,
}

#[cfg(test)]
impl HttpResponse for MockHttpResponse {
    fn status(&self) -> reqwest::StatusCode {
        self.status
    }

    fn deserialize_json<T: serde::de::DeserializeOwned>(&mut self) -> DuckResult<T> {
        if let Some(json) = &self.body {
            let result: T = serde_json::from_str(&json[..])?;
            return Ok(result);
        }
        return Err(format_err!("Response have no body!"));
    }

    fn headers(&self) -> &reqwest::header::HeaderMap {
        &self.headers
    }

    fn body(&mut self) -> DuckResult<String> {
        if let Some(text) = &self.body {
            return Ok(text.clone());
        }
        return Err(format_err!("Response have no body!"));
    }
}

#[cfg(test)]
impl HttpClient for MockHttpClient {
    type Item = MockHttpResponse;
    fn send(&self, request: &HttpRequestBuilder) -> DuckResult<MockHttpResponse> {
        let mut foo = self.request.lock().unwrap();
        foo.push(request.clone());

        let responses = self.responses.lock().unwrap();
        let response = responses.get(&request.url);
        if response.is_none() {
            return Err(format_err!("could not find expecation"));
        }

        Ok(response.unwrap().clone())
    }
}

#[cfg(test)]
pub struct MockHttpResponseBuilder {
    pub url: String,
    pub method: HttpMethod,
    pub status: Option<StatusCode>,
    pub body: Option<String>,
}

#[cfg(test)]
impl MockHttpResponseBuilder {
    pub fn new<T: Into<String>>(method: HttpMethod, url: T) -> Self {
        Self {
            url: url.into(),
            method,
            status: Some(StatusCode::OK),
            body: None,
        }
    }

    pub fn returns_status(mut self, status: StatusCode) -> Self {
        self.status = Some(status);
        self
    }

    pub fn returns_body<T: Into<String>>(mut self, json: T) -> Self {
        self.body = Some(json.into());
        self
    }

    pub fn build(self) -> DuckResult<MockHttpResponse> {
        if self.status.is_none() {
            return Err(format_err!("Status is not setup for expectation."));
        }
        Ok(MockHttpResponse {
            url: self.url,
            method: self.method,
            status: self.status.unwrap(),
            body: self.body,
            headers: reqwest::header::HeaderMap::new(),
        })
    }
}
