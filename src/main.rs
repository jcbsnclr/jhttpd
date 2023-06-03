#![feature(never_type)]

mod http;

use std::io;
use std::str::FromStr;

use tokio::io::{AsyncReadExt, AsyncBufReadExt, AsyncWriteExt};

async fn handle_stream(stream: tokio::net::TcpStream) -> io::Result<()> {
    let reader = tokio::io::BufReader::new(stream);
    let mut stream = reader.lines();

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = tokio::net::TcpListener::bind("localhost:1234").await?;

    loop {
        let (stream, addr) = server.accept().await?;

        let (reader, mut writer) = stream.into_split();

        let mut reader = tokio::io::BufReader::new(reader);
        println!("connection received from: {}", addr);

        let fut = async move {
            let req = http::Request::from_buf_reader(&mut reader).await;

            match req {
                Ok(req) => {
                    println!("got request: {req:#?}");
                    writer.write(b"HTTP/1.1 200 OK\r\n\r\nHello, Mum!").await
                        .map(|_| (()))
                },

                Err(err) => {
                    eprintln!("error: {err}; aborting connection");
                    Err(err)
                }
            }
        };

        tokio::spawn(fut);
    }

    Ok(())
}
