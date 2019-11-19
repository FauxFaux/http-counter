use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use failure::Error;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response,
};

async fn router(
    req: Request<Body>,
    state: Arc<Mutex<HashMap<String, u64>>>,
) -> Result<Response<Body>, hyper::Error> {
    let inc = "/inc/";
    Ok(match (req.method(), req.uri().path()) {
        (&Method::GET, "/status") => response(200, format!("{:?}", state.lock())),
        (&Method::GET, path) if path.starts_with(inc) => {
            let mut guard = state.lock().unwrap();
            let key = &path[inc.len()..];
            let current = guard.entry(key.to_string()).or_insert(0);
            *current += 1;
            response(200, "k")
        }
        _ => response(404, "404"),
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let state = Mutex::new(HashMap::new());
    let state = Arc::new(state);

    let addr = "127.0.0.1:1773".parse()?;

    let server = hyper::Server::bind(&addr).serve(make_service_fn(move |_| {
        let state = Arc::clone(&state);
        async { Ok::<_, hyper::Error>(service_fn(move |req| router(req, Arc::clone(&state)))) }
    }));

    server.await?;

    Ok(())
}

pub fn response<S: ToString>(status: u16, body: S) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(body.to_string().into())
        .expect("static builder")
}
