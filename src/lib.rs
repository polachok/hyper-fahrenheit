//! Have you ever wondered if you could run [hyper](https://docs.rs/hyper) on
//! [fahrenheit](https://docs.rs/fahrenheit/)?
//! I bet you haven't, but yes, you can (but please don't).
//!
//! ## Example:
//! ```
//! use fahrenheit;
//! use hyper::{Client, Uri};
//! use hyper_fahrenheit::{Connector, FahrenheitExecutor};
//!
//! fahrenheit::run(async move {
//!   let client: Client<Connector, hyper::Body> = Client::builder()
//!       .executor(FahrenheitExecutor)
//!       .build(Connector);
//!   let res = client
//!       .get(Uri::from_static("http://httpbin.org/ip"))
//!       .await
//!       .unwrap();
//!   println!("status: {}", res.status());
//!   let buf = hyper::body::to_bytes(res).await.unwrap();
//!   println!("body: {:?}", buf);
//! ```

use futures_io::{AsyncRead, AsyncWrite};
use futures_util::future::BoxFuture;
use hyper::rt::Executor;
use hyper::{
    client::connect::{Connected, Connection},
    service::Service,
    Uri,
};
use std::io::Error;
use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Wraps fahrenheit's TcpStream for hyper's pleasure.
pub struct AsyncTcpStream(fahrenheit::AsyncTcpStream);

impl AsyncTcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<AsyncTcpStream, std::io::Error> {
        Ok(AsyncTcpStream(fahrenheit::AsyncTcpStream::connect(addr)?))
    }
}

// Hyper needs this.
impl tokio::io::AsyncRead for AsyncTcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        ctx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        let this = Pin::into_inner(self);
        AsyncRead::poll_read(Pin::new(&mut this.0), ctx, buf)
    }
}

impl tokio::io::AsyncWrite for AsyncTcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        ctx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        let this = Pin::into_inner(self);
        AsyncWrite::poll_write(Pin::new(&mut this.0), ctx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        let this = Pin::into_inner(self);
        AsyncWrite::poll_flush(Pin::new(&mut this.0), cx)
    }
    fn poll_shutdown(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Connector;

impl Service<Uri> for Connector {
    type Response = AsyncTcpStream;
    type Error = std::io::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&mut self, req: Uri) -> Self::Future {
        let fut = async move {
            let addr = format!("{}:{}", req.host().unwrap(), req.port_u16().unwrap_or(80));
            AsyncTcpStream::connect(addr)
        };

        Box::pin(fut)
    }

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl Connection for AsyncTcpStream {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

// Wraps fahrenheit as hyper's Executor.
pub struct FahrenheitExecutor;

impl<Fut> Executor<Fut> for FahrenheitExecutor
where
    Fut: Send + std::future::Future<Output = ()> + 'static,
{
    fn execute(&self, fut: Fut) {
        fahrenheit::spawn(fut);
    }
}
