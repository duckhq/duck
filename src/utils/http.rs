use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;

use reqwest::{Client, Response, StatusCode};

use crate::utils::DuckResult;

pub trait HttpClient: Send + Sync {
    type Item: HttpResponse;
    fn send(&self, request: &HttpRequestBuilder) -> DuckResult<Self::Item>;
}

#[derive(Clone, PartialEq, Debug)]
pub enum HttpMethod {
    Post,
    Put,
}

pub trait HttpResponse {
    fn status(&self) -> StatusCode;
    fn get_json<T: serde::de::DeserializeOwned>(&mut self) -> DuckResult<T>;
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
    fn get_json<T: serde::de::DeserializeOwned>(&mut self) -> DuckResult<T> {
        let result: T = self.json()?;
        Ok(result)
    }
}

impl HttpClient for ReqwestClient {
    type Item = Response;

    fn send(&self, request: &HttpRequestBuilder) -> DuckResult<Response> {
        let mut builder = match &request.method {
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

        // let foo = builder.send()?;
        // foo.json()

        Ok(builder.send()?)
    }
}

#[cfg(test)]
pub struct MockHttpClient {
    pub expectations: Mutex<HashMap<String, MockHttpClientExpectation>>,
    pub request: Mutex<Vec<HttpRequestBuilder>>,
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
            expectations: Mutex::new(HashMap::new()),
            request: Mutex::new(Vec::new()),
        }
    }

    pub fn add_expectation(&self, builder: MockHttpClientExpectationBuilder) {
        let mut expectations = self.expectations.lock().unwrap();
        let expectation = builder.build().unwrap();
        expectations.insert(expectation.url.clone(), expectation);
    }

    pub fn get_sent_requests(&self) -> Vec<HttpRequestBuilder> {
        let requests = self.request.lock().unwrap();
        requests.clone()
    }
}

#[cfg(test)]
pub struct MockHttpResponse {
    status: StatusCode,
}

#[cfg(test)]
impl HttpResponse for MockHttpResponse {
    fn status(&self) -> reqwest::StatusCode {
        self.status
    }

    fn get_json<T: serde::de::DeserializeOwned>(&mut self) -> DuckResult<T> {
        unimplemented!()
    }
}

#[cfg(test)]
impl HttpClient for MockHttpClient {
    type Item = MockHttpResponse;
    fn send(&self, request: &HttpRequestBuilder) -> DuckResult<MockHttpResponse> {
        let mut foo = self.request.lock().unwrap();
        foo.push(request.clone());

        let expectations = self.expectations.lock().unwrap();
        let expecation = expectations.get(&request.url);
        if expecation.is_none() {
            return Err(format_err!("could not find expecation"));
        }

        let expecation = expecation.unwrap();
        Ok(MockHttpResponse {
            status: expecation.status,
        })
    }
}

#[cfg(test)]
pub struct MockHttpClientExpectation {
    pub url: String,
    pub method: HttpMethod,
    pub status: StatusCode,
}

#[cfg(test)]
pub struct MockHttpClientExpectationBuilder {
    pub url: String,
    pub method: HttpMethod,
    pub status: Option<StatusCode>,
}

#[cfg(test)]
impl MockHttpClientExpectationBuilder {
    pub fn new<T: Into<String>>(method: HttpMethod, url: T, status: StatusCode) -> Self {
        Self {
            url: url.into(),
            method,
            status: Some(status),
        }
    }

    pub fn build(self) -> DuckResult<MockHttpClientExpectation> {
        if self.status.is_none() {
            return Err(format_err!("Status is not setup for expectation."));
        }
        Ok(MockHttpClientExpectation {
            url: self.url,
            method: self.method,
            status: self.status.unwrap(),
        })
    }
}
