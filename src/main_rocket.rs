use std::process::Command;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use database::Database;
use rocket::futures::SinkExt;
use rocket::futures::StreamExt;
use rocket::get;
use rocket::http::ContentType;
use rocket::response::stream::Event;
use rocket::response::stream::EventStream;
use rocket::routes;
use rocket::tokio::sync::Mutex;
use rocket::FromFormField;
use serde::Serialize;
use util::unix_to_apple;

mod database;
mod structs;
mod util;
const UNAUTHORIZED: &str = "{\"status\":401,\"message\":\"You are not authorized to access this resource\",\"error\":{\"type\":\"Authentication Error\",\"message\":\"Unauthorized\"}}";

struct State<'a> {
    database: Mutex<Database>,
    password: &'a str,
}

#[get("/")]
async fn home_page() -> (ContentType, &'static str) {
    (ContentType::HTML, include_str!("homepage.html"))
}

#[derive(Debug, PartialEq, FromFormField)]
enum ChatExtras {
    Participants,
    LastMessage,
}

#[get("/chat/<chat_guid>?<password>&<with>")]
async fn get_chat_by_guid(state: &rocket::State<State<'_>>, chat_guid: &str, password: &str, with: Vec<ChatExtras>) -> (ContentType, String) {
    if state.password == password {
        let chat = state.database.lock().await.get_chat_by_guid(chat_guid.to_string(), with.contains(&ChatExtras::LastMessage), with.contains(&ChatExtras::Participants));
        return (ContentType::JSON, wrap_success(serde_json::to_string(&chat).unwrap()));
    }
    return (ContentType::JSON, UNAUTHORIZED.to_owned())
}

#[derive(Debug, PartialEq, FromFormField)]
enum ChatMessageExtras {
    Attachment,
    Handle,
    //SMS is not supported rn cause I said so
}

#[get("/chat/<chat_guid>/message?<password>&<with>&<after>&<before>&<offset>&<limit>&<sort>")]
async fn get_chat_messages(state: &rocket::State<State<'_>>, chat_guid: &str, password: &str, with: Vec<ChatMessageExtras>, after: Option<u128>, before: Option<u128>, offset: Option<u32>, limit: Option<u32>, sort: &str) -> Option<(ContentType, String)> {
    if state.password == password {
        let after_apple = unix_to_apple(after.unwrap_or(0)*1000000000);
        let before_apple = unix_to_apple(before.unwrap_or(0)*1000000000);
        if sort != "ASC" && sort != "DESC" {
            return None;
        }
        let messages = state.database.lock().await.get_chat_messages(chat_guid.to_string(), with.contains(&ChatMessageExtras::Attachment), with.contains(&ChatMessageExtras::Handle), offset.unwrap_or(0), limit.unwrap_or(1000), sort, after_apple, before_apple);
        return Some((ContentType::JSON,wrap_success(serde_json::to_string(&messages).unwrap())));
    }
    return Some((ContentType::JSON, UNAUTHORIZED.to_owned()))
}

#[derive(Serialize)]
struct ServerInfo<'a> {
    os_version: String,
    server_version: &'a str,
    private_api: bool,
    proxy_service: &'a str,
    helper_connected: bool,
}

#[get("/server/info?<guid>")]
async fn server_info(guid: &str, state: &rocket::State<State<'_>>) -> (ContentType, String) {
    if guid == state.password {
        return (ContentType::JSON, serde_json::to_string(&ServerInfo {
            os_version: String::from_utf8(Command::new("sw_vers").arg("productVersion").output().unwrap().stdout).unwrap(),
            server_version: "0.0.1-rust",
            private_api: false,
            proxy_service: "Dynamic DNS",
            helper_connected: false,
        }).unwrap());
    }
    (ContentType::JSON, UNAUTHORIZED.to_string())
}

#[get("/fcm/client?<guid>")]
async fn fcm_client_info(guid: &str, state: &rocket::State<State<'_>>) -> (ContentType, String) {
    if guid == state.password {
        return (ContentType::JSON, wrap_status("{}".to_string(), 200, "Successfully got FCM data".to_string()));
    }
    (ContentType::JSON, UNAUTHORIZED.to_string())
}

#[get("/socket.io?<guid>&<EIO>&<transport>")]
async fn socket_connection(ws: ws::WebSocket, state: &rocket::State<State<'_>>, guid: &str, EIO: u32, transport: &str) -> ws::Channel<'static> {
    ws.channel(move |mut stream| Box::pin(async move {
        while let Some(message) = stream.next().await {
            println!("{message:?}");
        }
        Ok(())
    }))
}

fn wrap_success(json: String) -> String {
    return format!("{{\"status\": 200, \"message\": \"Success\", \"data\": {}}}", json);
}

fn wrap_status(json: String, code: u32, message: String) -> String {
    return format!("{{\"status\": {}, \"message\": \"{}\", \"data\": {}}}", code, message, json);
}


#[rocket::main]
async fn main() {
    let database = Mutex::new(Database::new(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    let state = State {
        database,
        password: "balls",
    };
    let _ = rocket::build()
    .manage(state)
    .mount(
        "/",
        routes![home_page, socket_connection],
    )
    .mount(
        "/api/v1/",
        routes![get_chat_by_guid, get_chat_messages, server_info, fcm_client_info],
    )
    .launch()
    .await;
}

