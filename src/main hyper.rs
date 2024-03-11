use std::{convert::Infallible, net::SocketAddr, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use database::Database;
use http_body_util::Full;
use hyper::{body::Bytes, header::HeaderValue, server::conn::http1, service::service_fn, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde_json::Value;
use socketioxide::extract::{AckSender, Bin, Data, SocketRef};
use url::Url;
use tokio::{net::TcpListener, sync::Mutex};

mod database;
mod structs;
mod util;
const UNAUTHORIZED: &str = "{\"status\":401,\"message\":\"You are not authorized to access this resource\",\"error\":{\"type\":\"Authentication Error\",\"message\":\"Unauthorized\"}}";

fn socket_conn(socket: SocketRef, Data(data): Data<Value>) {
    println!("Socket.IO connected: {:?} {:?}", socket.ns(), socket.id);
    socket.emit("auth", data).ok();

    socket.on(
        "message",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
            socket.bin(bin).emit("message-back", data).ok();
        },
    );

    socket.on(
        "message-with-ack",
        |Data::<Value>(data), ack: AckSender, Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
            ack.bin(bin).send(data).ok();
        },
    );
}

struct State<'a> {
    database: Mutex<Database>,
    password: &'a str,
}

async fn router(state: Arc<State<'_>>, req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let path_and_query = req.uri().clone().into_parts().path_and_query.unwrap();
    let url = Url::options().base_url(Url::parse("http://0.0.0.0").ok().as_ref()).parse(&req.uri().to_string()).unwrap();
    let path: Vec<&str> = path_and_query.path()[1..].split("/").collect();
    if path.len() == 4 && path[0] == "api" && path[1] == "v1" && path[2] == "chat" {
        if let Some(password) = url.query_pairs().find_map(|pair| { if pair.0 == "password" { Some(pair.1) } else { None } }) {
            if password == state.password {
                let with = url.query_pairs().find_map(|pair| { if pair.0 == "with" { Some(pair.1) } else { None } });
                let (last_message, participants) = with.map(|with| { (with.contains("lastmessage"), with.contains("participants")) }).unwrap_or((false, false));
                return get_chat_by_guid(&state, path[3], &*password, last_message, participants).await;
            }
        }
    }
    let mut res = Response::new(Full::new(Bytes::from("Not Found")));
    *res.status_mut() = StatusCode::NOT_FOUND;
    Ok(res)
}

async fn get_chat_by_guid(state: &State<'_>, chat_guid: &str, password: &str, last_message: bool, participants: bool) -> Result<Response<Full<Bytes>>, Infallible> {
    if state.password == password {
        let chat = state.database.lock().await.get_chat_by_guid(chat_guid.to_string(), last_message, participants);
        let res = json_res(wrap_success(serde_json::to_string(&chat).unwrap()));
        return Ok(res)
    }
    return Ok(json_res(UNAUTHORIZED.into()));
}

fn json_res(body: String) -> Response<Full<Bytes>> {
    let mut res = Response::new(Full::new(Bytes::from(body)));
    res.headers_mut().insert("Content-Type", HeaderValue::from_str("application/json").unwrap());
    res
}

fn wrap_success(json: String) -> String {
    format!("{{\"status\": 200, \"message\": \"Success\", \"data\": {}}}", json)
}

fn _wrap_status(json: String, code: u32, message: String) -> String {
    format!("{{\"status\": {}, \"message\": \"{}\", \"data\": {}}}", code, message, json)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;
    
    let database = Mutex::new(Database::new(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    let state = State {
        database,
        password: "balls",
    };
    let state_arc = Arc::new(state);

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);
        let state_arc = state_arc.clone();

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(|req| {
                    return router(state_arc.clone(), req);
                }))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}