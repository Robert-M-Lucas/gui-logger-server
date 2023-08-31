#![feature(proc_macro_hygiene, decl_macro)]

mod websocket;
mod webserver;

#[macro_use] extern crate rocket;

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;

type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
pub async fn main() {
    let args: Vec<String> = env::args().collect();
    let (web_port, socket_port): (u16, u16) = if args.len() <= 1 {
        (8000, 8080)
    }
    else if args.len() != 3 {
        println!("Either specify 2 ports (website, socket) or specify neither for defaults to be used");
        return;
    }
    else {
        // Ugly one-liner because I felt like it
        let result = args[1].parse::<u16>().and_then(|web| args[2].parse::<u16>().and_then(|socket| Ok((web, socket))));
        if let Ok(ports) = result {
            ports
        }
        else {
            println!("Either specify 2 ports (website, socket) or specify neither for defaults to be used");
            return;
        }
    };

    println!("Using ports {} (website) and {} (socket)", web_port, socket_port);

    let users = Users::default();

    websocket::run_websocket(users.clone(), socket_port).await;

    webserver::run_webserver_blocking(users, web_port, socket_port);
}