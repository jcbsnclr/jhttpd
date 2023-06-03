use std::{str::FromStr, collections::HashMap, io};

use thiserror::Error;

use tokio::io::AsyncBufReadExt;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Options,
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Connect
}

impl FromStr for Method {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OPTIONS" => Ok(Method::Options),
            "GET"     => Ok(Method::Get),
            "HEAD"    => Ok(Method::Head),
            "POST"    => Ok(Method::Post),
            "PUT"     => Ok(Method::Put),
            "DELETE"  => Ok(Method::Delete),
            "TRACE"   => Ok(Method::Trace),
            "CONNECT" => Ok(Method::Connect),
            
            s   => Err(ParserError::InvalidMethod)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    method: Method,
    url: Url,
    headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Error)]
pub enum ParserError {
    #[error("invalid protocol version")]
    InvalidProtocol,
    #[error("invalid method")]
    InvalidMethod,

    #[error("unexpected EOF")]
    UnexpectedEof
}

fn to_io_error<E>(error: E) -> io::Error
where E: Into<Box<dyn std::error::Error + Send + Sync>> {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

impl Request {
    pub async fn from_buf_reader<B>(stream: &mut B) -> io::Result<Request>
    where B: AsyncBufReadExt + Sized + Unpin {
        // split the incoming HTTP request into lines for processing
        let mut lines = stream.lines();

        // extract start line
        let start_line = lines.next_line().await?
            .ok_or(to_io_error(ParserError::UnexpectedEof))?;

        // split header line into words for parsing
        let mut words = start_line.split(' ');

        // extract method
        let method = words.next()
            .ok_or(ParserError::UnexpectedEof)
            .and_then(|m| Method::from_str(m))
            .map_err(to_io_error)?;

        // extract URL
        let url = words.next()
            .ok_or(ParserError::UnexpectedEof)
            .map_err(to_io_error)
            .and_then(|u| {
                let base = url::Url::parse("http://localhost/");

                base.and_then(|b| b.join(u))
                    .map_err(to_io_error)
            })?;

        // make sure protocol is specified as "HTTP/1.1"
        words.next()
            .ok_or(ParserError::UnexpectedEof)
            .and_then(|p| if p == "HTTP/1.1" {
                Ok(())
            } else {
                Err(ParserError::InvalidProtocol)
            })
            .map_err(to_io_error)?;

        let headers = HashMap::new();

        Ok(Request {
            method,
            url,
            headers
        })
    }
}