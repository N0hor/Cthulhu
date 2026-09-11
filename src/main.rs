// AI-assisted pseudocode driven development
// Commented and adapted by a human

use serde_yaml::{from_reader, Value};
use std::fs::File;
use std::io::Read;
use tiny_http::{Server, Request};

const CONFIG_PATH: &str = "conf.yml";

fn main() {
    // Load and parse configuration
    let conf = load_conf();
    
    let entry_port = conf["entry_port"]
        .as_u64()
        .unwrap()
        as u16;

    let redirect_port = conf["port_to_redirect_to_after_verification"]
        .as_u64()
        .unwrap()
        as u16;

    let paths = conf["paths"].clone();

    // Proxy loop : intercept the request, validate its authorization, and forward it
    let _ = proxy_loop(entry_port, redirect_port, &paths);
}

fn load_conf() -> Value {
    let conf_file = File::open(CONFIG_PATH)
        .expect("Configuration file not found.");

    let yml_content = from_reader(conf_file)
        .expect("Can't read configuration file.");

    return yml_content;
}

fn proxy_loop(entry_port: u16, redirect_port: u16, _paths: &Value) {

    let server = Server::http(format!("127.0.0.1:{}", entry_port)).unwrap();
    println!("Proxy listening on {}", entry_port);

    for mut request in server.incoming_requests() {
        debug(&mut request);
    }
}

fn debug(request: &mut Request) {
    let mut body = Vec::new();
    request.as_reader().read_to_end(&mut body).unwrap();

    let body = String::from_utf8_lossy(&body);

    println!(
        "{} {} {:?} {}",
        request.method(),
        request.url(),
        request.headers(),
        body
    );
}