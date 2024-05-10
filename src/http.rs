use strum_macros::{AsRefStr, EnumString};

use crate::{Error, Result};
use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
    str::FromStr,
};

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub version: HttpVersion,
    pub headers: Vec<HttpHeader>,
    pub body: Option<HttpBody>,
}

impl TryFrom<&mut BufReader<TcpStream>> for HttpRequest {
    type Error = Error;

    fn try_from(reader: &mut BufReader<TcpStream>) -> Result<Self> {
        let mut s = String::new();
        reader.read_line(&mut s)?;

        if !s.ends_with("\r\n") {
            Err(Error::MissingCRLFFromLine)?;
        }
        s.pop();
        s.pop();

        // example first line: GET /index.html HTTP/1.1
        let parts: Vec<_> = s.split(' ').collect();
        if parts.len() != 3 {
            Err(Error::InvalidRequestLine(s.clone()))?;
        }
        let method = HttpMethod::from_str(parts[0])?;
        let path = parts[1].to_string();
        let version = HttpVersion::from_str(parts[2])?;

        let mut headers = Vec::new();
        loop {
            let mut s = String::new();
            reader.read_line(&mut s)?;
            if s.len() <= 2 {
                break;
            }

            let header = HttpHeader::try_from(s)?;

            headers.push(header);
        }

        Ok(HttpRequest {
            method,
            path,
            version,
            headers,
            body: None,
        })
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: HttpStatus,
    pub version: HttpVersion,
    pub headers: Vec<HttpHeader>,
    pub body: Option<HttpBody>,
}
impl From<HttpResponse> for String {
    fn from(response: HttpResponse) -> Self {
        let mut val = format!(
            "{} {}\r\n",
            response.version.as_ref(),
            response.status.as_ref()
        );

        for header in response.headers {
            val.push_str(String::from(header).as_ref());
        }
        val.push_str("\r\n");
        match response.body {
            None => {}
            Some(body) => {
                val.push_str(String::from(body).as_ref());
            }
        }
        val.push_str("\r\n");

        val
    }
}

impl HttpResponse {
    pub fn empty_response() -> Self {
        HttpResponse {
            status: HttpStatus::Ok200,
            version: HttpVersion::V1_1,
            // https://datatracker.ietf.org/doc/html/rfc7230#section-3.3
            // good practice to add a content length header
            headers: vec![HttpHeader {
                key: "Content-Length".to_string(),
                value: "0".to_string(),
            }],
            body: None,
        }
    }
    pub fn not_found_response() -> Self {
        HttpResponse {
            status: HttpStatus::NotFound404,
            version: HttpVersion::V1_1,
            headers: vec![HttpHeader {
                key: "Content-Length".to_string(),
                value: "0".to_string(),
            }],
            body: None,
        }
    }
    pub fn plain_text_response(content: &str) -> Self {
        let headers = vec![
            HttpHeader {
                key: "Content-Type".to_string(),
                value: "text/plain".to_string(),
            },
            HttpHeader {
                key: "Content-Length".to_string(),
                value: content.len().to_string(),
            },
        ];

        HttpResponse {
            status: HttpStatus::Ok200,
            version: HttpVersion::V1_1,
            headers,
            body: Some(HttpBody::Text(content.to_string())),
        }
    }

    pub fn file_response(content: &str) -> Self {
        let headers = vec![
            HttpHeader {
                key: "Content-Type".to_string(),
                value: "application/octet-stream".to_string(),
            },
            HttpHeader {
                key: "Content-Length".to_string(),
                value: content.len().to_string(),
            },
        ];

        HttpResponse {
            status: HttpStatus::Ok200,
            version: HttpVersion::V1_1,
            headers,
            body: Some(HttpBody::Text(content.to_string())),
        }
    }
}

#[derive(EnumString, AsRefStr, Debug)]
pub enum HttpMethod {
    #[strum(serialize = "GET", ascii_case_insensitive)]
    Get,
}

#[derive(EnumString, AsRefStr, Debug)]
pub enum HttpVersion {
    #[strum(serialize = "HTTP/1.1", ascii_case_insensitive)]
    V1_1,
}

#[derive(AsRefStr, Debug)]
pub enum HttpStatus {
    #[strum(serialize = "200 OK")]
    Ok200,
    #[strum(serialize = "404 Not Found")]
    NotFound404,
}

#[derive(Debug)]
pub struct HttpHeader {
    pub key: String,
    pub value: String,
}

impl TryFrom<String> for HttpHeader {
    type Error = Error;

    fn try_from(line: String) -> Result<Self> {
        if !line.ends_with("\r\n") {
            Err(Error::MissingCRLFFromLine)?;
        }

        let (key, value) = line[..line.len() - 2]
            .split_once(':')
            .ok_or_else(|| Error::InvalidHeader)?;
        Ok(Self {
            key: key.trim().to_string(),
            value: value.trim().to_string(),
        })
    }
}

impl From<HttpHeader> for String {
    fn from(header: HttpHeader) -> Self {
        format!("{}: {}\r\n", header.key, header.value)
    }
}

#[derive(Debug)]
pub enum HttpBody {
    Text(String),
}

impl From<HttpBody> for String {
    fn from(body: HttpBody) -> Self {
        match body {
            HttpBody::Text(x) => x,
        }
    }
}
