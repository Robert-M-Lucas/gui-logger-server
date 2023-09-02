use tokio::runtime::Runtime;
use rocket::{Data, Response, State};
use rocket::http::Status;
use serde::{Deserialize, Serialize};
use rocket_contrib::serve::StaticFiles;
use rocket::response::NamedFile;
use std::io::Read;
use std::thread;
use std::time::Duration;
use crate::{Users, websocket};

struct AppState {
    users: Users,
    runtime: Runtime,
    socket_port: u16
}

pub fn run_webserver_blocking(users: Users, port: u16, socket_port: u16, open_browser: bool) {
    let cfg = rocket::config::Config::build(rocket::config::Environment::Production)
        .address("0.0.0.0")
        .port(port)
        .unwrap();

    if open_browser {
        let port_clone = port.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            open::that(format!("http://0.0.0.0:{}", port_clone)).ok();
        });
    }

    rocket::custom(cfg)
        .mount("/", routes![root, send, port])
        .mount("/static", StaticFiles::from("web/static"))
        .manage(AppState { users: users.clone(), runtime: Runtime::new().unwrap(), socket_port })
        .launch();
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
fn send(state: State<AppState>, input: Data) -> Response {
    let mut input_string = String::new();
    input.open().read_to_string(&mut input_string).unwrap();
    let data = serde_json::from_str(&input_string).and_then(|d: GUIData| serde_json::to_string(&d));
    if let Ok(data) = data {
        #[cfg(debug_assertions)]
        println!("Received data, forwarding");
        state.runtime.spawn(websocket::user_message(data, state.users.clone()));
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

#[get("/port")]
fn port(state: State<AppState>) -> String {
    state.socket_port.to_string()
}