use strum_macros::{AsRefStr, EnumString};

use crate::{Error, Result};
use std::{
    io::{BufRead, BufReader, Read, Write},
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
        let mut content_length: usize = 0;

        loop {
            let mut s = String::new();
            reader.read_line(&mut s)?;
            if s.len() <= 2 {
                break;
            }

            let header = HttpHeader::try_from(s)?;

            if header.key.to_lowercase() == "content-length" {
                content_length = header.value.parse::<_>()?;
            }

            headers.push(header);
        }

        let http_body = {
            match content_length {
                0 => None,
                x => {
                    let mut body = vec![0; x];
                    reader.read_exact(&mut body)?;
                    let body = String::from_utf8(body)?;

                    Some(HttpBody::Text(body))
                }
            }
        };

        Ok(HttpRequest {
            method,
            path,
            version,
            headers,
            body: http_body,
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
impl From<HttpResponse> for Vec<u8> {
    fn from(response: HttpResponse) -> Self {
        let mut res = Vec::new();
        let val = format!(
            "{} {}\r\n",
            response.version.as_ref(),
            response.status.as_ref()
        );
        res.extend(val.as_bytes());

        for header in response.headers {
            res.extend::<Vec<u8>>(header.into());
        }
        res.extend(b"\r\n");
        match response.body {
            None => {}
            Some(body) => {
                let body_bytes: Vec<u8> = body.into();
                res.extend(body_bytes);
            }
        }

        res
    }
}

impl HttpResponse {
    pub fn empty_response(status: HttpStatus) -> Self {
        HttpResponse {
            status,
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

    /// accepted_encodings is a list of comma separated values
    pub fn content_response(
        content: &str,
        content_type: &str,
        accepted_encodings: &str,
    ) -> Result<Self> {
        let mut compression = String::from("None");
        for accepted_encoding in accepted_encodings.split(',') {
            if accepted_encoding.trim() == "gzip" {
                compression = String::from("gzip");
                break;
            }
        }

        let http_body = match compression.as_ref() {
            "gzip" => HttpBody::gzip_from_content(content)?,
            _ => HttpBody::from_content(content),
        };

        let mut headers = vec![
            HttpHeader {
                key: "Content-Type".to_string(),
                value: content_type.to_string(),
            },
            HttpHeader {
                key: "Content-Length".to_string(),
                value: http_body.content_length().to_string(),
            },
        ];

        match compression.as_ref() {
            "gzip" => {
                headers.push(HttpHeader {
                    key: "Content-Encoding".to_string(),
                    value: "gzip".to_string(),
                });
            }
            _ => {}
        };

        Ok(HttpResponse {
            status: HttpStatus::Ok200,
            version: HttpVersion::V1_1,
            headers,
            body: Some(http_body),
        })
    }
}

#[derive(EnumString, AsRefStr, Debug)]
pub enum HttpMethod {
    #[strum(serialize = "GET", ascii_case_insensitive)]
    Get,
    #[strum(serialize = "POST", ascii_case_insensitive)]
    Post,
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
    #[strum(serialize = "201 Created")]
    Created201,
    #[strum(serialize = "500 Internal Server Error")]
    InternalServerError500,
}

#[derive(Debug, Clone)]
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

impl From<HttpHeader> for Vec<u8> {
    fn from(header: HttpHeader) -> Self {
        Vec::from(format!("{}: {}\r\n", header.key, header.value).as_bytes())
    }
}

#[derive(Debug, Clone)]
pub enum HttpBody {
    Text(String),
    Gzip(Vec<u8>),
}

impl From<HttpBody> for Vec<u8> {
    fn from(body: HttpBody) -> Self {
        match body {
            HttpBody::Text(x) => Vec::from(x.as_bytes()),
            HttpBody::Gzip(x) => x,
        }
    }
}

impl HttpBody {
    pub fn content_length(&self) -> usize {
        match self {
            Self::Text(x) => x.as_bytes().len(),
            Self::Gzip(x) => x.len(),
        }
    }
    pub fn from_content(content: &str) -> Self {
        Self::Text(content.to_string())
    }

    pub fn gzip_from_content(content: &str) -> Result<Self> {
        let body_bytes = content.as_bytes();
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        e.write_all(body_bytes).unwrap();
        let encoded_bytes = e.finish()?;
        Ok(Self::Gzip(encoded_bytes))
    }
}
