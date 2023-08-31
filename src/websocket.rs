use warp::ws::{Message, WebSocket};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use warp::Filter;
use crate::Users;


pub async fn run_websocket(users_clone: Users, port: u16) {
    let users_warp = warp::any().map(move || users_clone.clone());

    let register = warp::path("ws")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(users_warp)
        .map(|ws: warp::ws::Ws, users| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, users))
        });

    let warp_future = warp::serve(register).run(([127, 0, 0, 1], port));
    tokio::spawn(warp_future);
}

pub async fn user_connected(ws: WebSocket, users: Users) {
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

pub async fn user_message(msg: String, users: Users) {
    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        #[cfg(debug_assertions)]
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

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(0);
