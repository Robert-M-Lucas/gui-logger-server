#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::{Arc};
use std::sync::atomic::{AtomicUsize, Ordering};
use rocket::response::NamedFile;
use rocket_contrib::serve::StaticFiles;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::Filter;
use warp::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use rocket::{Response, response, State};
use rocket::http::Status;
use tokio::runtime::Runtime;
use serde::{Serialize, Deserialize};

type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(0);
static GLOBAL_RUNTIME: RefCell<Option<Runtime>> = RefCell::new(None);

struct AppState {
    users: Users
}

#[get("/")]
fn root() -> NamedFile {
    NamedFile::open("web/main.html").unwrap()
}

#[derive(Serialize, Deserialize)]
struct GUIData {
    path: String,
    r#type: String,
    data: String,
}

#[put("/send", data="<input>")]
fn send(state: State<AppState>, input: String) -> Response {
    let data = serde_json::from_str(&input).and_then(|d: GUIData| serde_json::to_string(&d));
    if let Ok(data) = data {
        println!("Recieved data, forwarding");
        GLOBAL_RUNTIME.borrow().unwrap().spawn(user_message(data, state.users.clone()));
        let mut response = Response::new();
        response.set_status(Status::Ok);
        response
    }
    else {
        println!("Received data, not forwarding due to improper formatting");
        let mut response = Response::new();
        response.set_status(Status::BadRequest);
        response
    }
}

#[tokio::main]
async fn main() {
    *GLOBAL_RUNTIME.borrow_mut().unwrap() = Runtime::new().unwrap();
    let users = Users::default();
    let users_clone = users.clone();
    let users_warp = warp::any().map(move || users_clone.clone());

    let register = warp::path("ws")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(users_warp)
        .map(|ws: warp::ws::Ws, users| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, users))
        });

    let warp_future = warp::serve(register).run(([127, 0, 0, 1], 8080));
    tokio::spawn(warp_future);

    rocket::ignite()
        .mount("/", routes![root, send])
        .mount("/static", StaticFiles::from("web/static"))
        .manage(AppState { users: users.clone() })
        .launch();
}

async fn user_connected(ws: WebSocket, users: Users) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    println!("New websocket connection: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    println!("Websocket [{}] send error: {}", my_id, e);
                })
                .await;
        }
    });

    // Save the sender in our list of connected users.
    users.write().await.insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    while let Some(result) = user_ws_rx.next().await {
        let _msg = match result {
            Ok(msg) => println!("Websocket [{}] message received: {:?}", my_id, msg),
            Err(e) => {
                println!("Websocket [{}] error: {}", my_id, e);
                break;
            }
        };
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users).await;
}

async fn user_message(msg: String, users: Users) {
    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        println!("Sending data to user: {}", uid);
        if let Err(_disconnected) = tx.send(Message::text(msg.clone())) {
            // The tx is disconnected, our `user_disconnected` code
            // should be happening in another task, nothing more to
            // do here.
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    println!("Web socket connection lost: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}