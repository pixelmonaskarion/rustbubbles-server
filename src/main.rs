use std::{collections::HashMap, process::Command, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use database::Database;
use hyper::{Response, StatusCode, Uri};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socketioxide::{extract::{AckSender, Bin, Data, SocketRef}, SocketIo};
use tokio::sync::Mutex;
use axum::{body::Body, extract::Query, http::HeaderValue, response::IntoResponse, routing::{get, post}, Json};
use axum::extract::Path;

use crate::util::unix_to_apple;

mod database;
mod structs;
mod util;
const UNAUTHORIZED: &str = "{\"status\":401,\"message\":\"You are not authorized to access this resource\",\"error\":{\"type\":\"Authentication Error\",\"message\":\"Unauthorized\"}}";

fn socket_conn(socket: SocketRef) {
    
    println!("Socket.IO connected: {:?} {:?} {:?}", socket.ns(), socket.id, socket.transport_type());

    socket.on(
        "get-server-metadata",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "save-vcf",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "get-vcf",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "change-proxy-service",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "get-server-config",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "add-fcm-device",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "get-fcm-client",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "get-logs",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "get-chats",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "get-chat",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "get-chat-messages",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "get-messages",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "get-attachment",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "get-attachment-chunk",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "get-last-chat-message",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "get-participants",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "send-message",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "send-message-chunk",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
    socket.on(
        "get-contacts-from-vcf",
        |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        },
    );
}

async fn fallback(uri: Uri) -> (StatusCode, String) {
    println!("client requested unknown page {uri}");
    (StatusCode::NOT_FOUND, format!("No route for {uri}"))
}

struct State<'a> {
    database: Mutex<Database>,
    password: &'a str,
}

#[derive(Deserialize, Debug)]
struct ChatQuery {
    limit: Option<usize>,
    offset: Option<usize>,
    with: Option<Vec<String>>,
    sort: Option<String>,
}

#[derive(Serialize)]
struct ServerInfo<'a> {
    os_version: String,
    server_version: &'a str,
    private_api: bool,
    proxy_service: &'a str,
    helper_connected: bool,
    detected_icloud: String,
}

fn wrap_success(json: String) -> Response<Body> {
    let mut res = format!("{{\"status\": 200, \"message\": \"Success\", \"data\": {}}}", json).into_response();
    res.headers_mut().insert("Content-Type", HeaderValue::from_str("application/json").unwrap());
    res
}

fn wrap_status(json: String, code: u32, message: String) -> String {
    return format!("{{\"status\": {}, \"message\": \"{}\", \"data\": {}}}", code, message, json);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (layer, io) = SocketIo::new_layer();
    let database = Mutex::new(Database::new(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    let state = State {
        database,
        password: "balls",
    };
    let state_chat_guid = Arc::new(state);
    let state_chat_query = state_chat_guid.clone();
    let state_server_info = state_chat_guid.clone();
    let state_contacts = state_chat_guid.clone();
    let state_chat_count = state_chat_guid.clone();
    let state_message_guid = state_chat_guid.clone();
    let state_chat_message = state_chat_guid.clone();

    // Register a handler for the default namespace
    io.ns("/", socket_conn);

    let app = axum::Router::new()
    .route("/api/v1/chat/:guid", get(|Path(guid): Path<String>, Query(params): Query<HashMap<String, String>>| async move {
        println!("{params:?}");
        let password = params.get("guid"); 
        if password.map(|password| password != state_chat_guid.password).unwrap_or(true) {
            return UNAUTHORIZED.to_string().into_response();
        }
        let with = params.get("with");
        let (last_message, participants) = with.map(|with| { (with.contains("lastmessage"), with.contains("participants")) }).unwrap_or((false, false));
        let chats = state_chat_guid.database.lock().await.get_chat_by_guid(guid, last_message, participants);
        return wrap_success(serde_json::to_string(&chats).unwrap());
    }))
    .route("/api/v1/chat/query", post(|Query(params): Query<HashMap<String, String>>, Json(query): Json<ChatQuery>| async move {
        println!("{query:?}");
        let password = params.get("guid"); 
        if password.map(|password| password != state_chat_query.password).unwrap_or(true) {
            return UNAUTHORIZED.to_string().into_response();
        }
        let with = query.with.unwrap_or(vec![]);
        let chats = state_chat_query.database.lock().await.query_chats(query.limit.unwrap_or(1000), query.offset.unwrap_or(0), query.sort, with.contains(&"lastmessage".to_string()), true /* clients expect participants even without specifying so */);
        return wrap_success(serde_json::to_string(&chats).unwrap());
    }))
    .route("/api/v1/chat/count", get(|Query(params): Query<HashMap<String, String>>| async move {
        let password = params.get("guid"); 
        if password.map(|password| password != state_chat_count.password).unwrap_or(true) {
            return UNAUTHORIZED.to_string().into_response();
        }
        let chat_count = state_chat_count.database.lock().await.get_chat_count();
        return wrap_success(serde_json::to_string(&chat_count).unwrap());
    }))
    .route("/api/v1/server/info", get(|Query(params): Query<HashMap<String, String>>| async move {
        println!("client got server info");
        let password = params.get("guid"); 
        if password.map(|password| password == state_server_info.password).unwrap_or(false) {
            let mut detected_icloud = String::from_utf8(Command::new("/usr/libexec/PlistBuddy").arg("-c").arg("print :Accounts:0:AccountID").arg(&format!("{}/Library/Preferences/MobileMeAccounts.plist", std::env::var("HOME").unwrap())).output().unwrap().stdout).unwrap();
            detected_icloud.remove(detected_icloud.len()-1);
            return wrap_success(serde_json::to_string(&ServerInfo {
                os_version: String::from_utf8(Command::new("sw_vers").arg("productVersion").output().unwrap().stdout).unwrap(),
                server_version: "0.0.1",
                private_api: false,
                proxy_service: "Dynamic DNS",
                helper_connected: false,
                detected_icloud,
            }).unwrap());
        }
        return UNAUTHORIZED.to_string().into_response();
    }))
    .route("/api/v1/contact", get(|Query(params): Query<HashMap<String, String>>| async move {
        println!("client asked for contacts!");
        let password = params.get("guid"); 
        if password.map(|password| password == state_contacts.password).unwrap_or(false) {
            return "{\"status\":200,\"message\":\"Success\",\"data\":[]}".into();
        }
        return UNAUTHORIZED.to_string();
    }))
    .route("/api/v1/message/:guid", get(|Path(guid): Path<String>, Query(params): Query<HashMap<String, String>>| async move {
        let password = params.get("guid"); 
        if password.map(|password| password != state_message_guid.password).unwrap_or(true) {
            return UNAUTHORIZED.to_string().into_response();
        }
        let with = params.get("with");
        let (chats, participants) = with.map(|with| { (with.contains("chats"), with.contains("participants")) }).unwrap_or((false, false));
        let message = state_message_guid.database.lock().await.get_message_by_guid(guid, chats, participants);
        return wrap_success(serde_json::to_string(&message).unwrap());
    }))
    .route("/api/v1/chat/:guid/message", get(|Path(guid): Path<String>, Query(params): Query<HashMap<String, String>>| async move {
        println!("chat messages {guid} {params:?}");
        let password = params.get("guid"); 
        if password.map(|password| password != state_chat_message.password).unwrap_or(true) {
            return UNAUTHORIZED.to_string().into_response();
        }
        let with = params.get("with");
        let (chats, participants) = with.map(|with| { (with.contains("chats"), with.contains("participants")) }).unwrap_or((false, false));
        let limit = params.get("limit").unwrap_or(&String::from("1000")).parse().unwrap();
        let offset = params.get("offset").unwrap_or(&String::from("0")).parse().unwrap();
        let sort = params.get("sort").map(|string| string.as_str()).unwrap_or("ASC");
        let after = if params.get("after") == Some(&"".to_string()) {
            0
        } else {
            unix_to_apple(params.get("after").unwrap_or(&String::from("0")).parse::<u128>().unwrap()*1000000)
        };
        let before = if params.get("before") == Some(&"".to_string()) {
            u128::MAX
        } else {
            unix_to_apple(params.get("before").map(|string| string.parse::<u128>().unwrap()*1000000).unwrap_or(u128::MAX))
        };
        let message = state_chat_message.database.lock().await.get_chat_messages(guid, chats, participants, limit, offset, sort, after, before);
        println!("{message:?}");
        return wrap_success(serde_json::to_string(&message).unwrap());
    }))
    // .route("/api/v1/fcm/client", get(|Query(params): Query<HashMap<String, String>>| async move {
    //     let password = params.get("guid"); 
    //     if password.map(|password| password == state_fcm_client.password).unwrap_or(false) {
    //         return "{\"data\":null, \"status\": 200, \"message\":\"Success\"}".into();
    //     }
    //     return UNAUTHORIZED.to_string();
    // }))
    .fallback(fallback)
    .layer(layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}