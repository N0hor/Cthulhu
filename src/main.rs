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
        handle_request();
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

fn handle_request(){
        /*
        PSEUDO CODE
        requested_path = request.path
        if requested_path in paths (from the conf){
            if requested_methode in paths allowed methode (There can be several authorized for the same route){
                if valid_content { // Use Rust blocks.
                    If it's a query parameter in a GET request, is that specific parameter allowed
                        and is the expected format validated ?
                    If it's a body parameter for a POST/PUT/PATCH request, is that parameter allowed
                        and is the expected format validated?
                    If it's a JSON body for a POST/PUT/PATCH request, the same applies.

                    Add any cases I might have overlooked.

                        println!("Request authorized");
                }
            }
        }
        println!("Unauthorized request");
        
        */
}