use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

async fn redirect(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
  let mut res = Response::new(Full::from(Bytes::from("")));
  *res.status_mut() = hyper::StatusCode::MOVED_PERMANENTLY;
  let uri = format!("https://{}{}", req.headers().get("host").unwrap().to_str().unwrap(), req.uri().path());
  res.headers_mut().insert(hyper::header::LOCATION, uri.parse().unwrap());
  Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let addr = SocketAddr::from(([0, 0, 0, 0], 80));

  let listener = TcpListener::bind(addr).await?;

  loop {
    let (stream, _) = listener.accept().await?;
    let io = TokioIo::new(stream);

    tokio::task::spawn(async move {
      let res = http1::Builder::new()
        .serve_connection(io, service_fn(redirect))
        .await;

      if let Err(err) = res {
        println!("Error serving connection: {:?}", err);
      }
    });
  }
}
