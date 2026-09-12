// AI-assisted pseudocode driven development
// Commented and adapted by a human

use serde_yaml::{from_reader, Value};
use std::fs::File;
use std::io::Read;
use tiny_http::{Server, Request};
use std::collections::HashMap;
use serde_json::Value as JsonValue;
use url::form_urlencoded;

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

    let redirect_host = conf["host_to_redirect_to_after_verification"]
        .as_str()
        .unwrap()
        .to_string();

    let paths = conf["paths"].clone();

    // Proxy loop : intercept the request, validate its authorization, and forward it
    let _ = proxy_loop(entry_port, &redirect_host, redirect_port, &paths);
}

fn load_conf() -> Value {
    let conf_file = File::open(CONFIG_PATH)
        .expect("Configuration file not found.");

    let yml_content = from_reader(conf_file)
        .expect("Can't read configuration file.");

    return yml_content;
}

fn proxy_loop(entry_port: u16, redirect_host: &str, redirect_port: u16, paths: &Value) {

    let server = Server::http(format!("0.0.0.0:{}", entry_port)).unwrap();
    println!("Cthulhu listening on {}", entry_port);

    for mut request in server.incoming_requests() {

        let body = {
            /* The request body is provided as a data stream, 
            so Rust gives a reader that we must read to retrieve its content. */
            let mut bytes = Vec::new();

            request.as_reader()
                .read_to_end(&mut bytes)
                .unwrap();

            String::from_utf8_lossy(&bytes).to_string()
        };

        handle_request(request, &body, paths, redirect_host, redirect_port);
    }
}

fn handle_request(request: Request, body: &str, paths: &Value, redirect_host: &str, redirect_port: u16) {

    // Get method
    let method = request.method().to_string();

    // Parse url : path + query
    let full_url = request.url();
    let requested_path;
    let query_string;

    if full_url.contains('?') {
        let parts: Vec<&str> = full_url.splitn(2, '?').collect();

        requested_path = parts[0];
        query_string = parts[1];
    } else {
        requested_path = full_url;
        query_string = "";
    }

    // Is requested_path match a path in the config ?
    let path_config = match paths.get(requested_path) {
        Some(value) => value, // if we match value yes !
        None => { println!("Unauthorized request"); return; } // else
    };

    // Is method allowed for this path in the config ?
    let method_config = match path_config.get(&method) {
        Some(value) => value,
        None => { println!("Unauthorized request"); return; }
    };

    // Query string validation
    match method_config.get("query") {
        Some(query_schema) => { // if there is a query parameter
            if !validate_query(query_string, query_schema) { // but not allowed in the conf
                println!("Unauthorized request");
                return;
            }
        }
        None => { // if their is a query parameter, but without content 
            if !query_string.is_empty() {
                println!("Unauthorized request");
                return;
            }
        }
    }

    // Body validation
    match method_config.get("body") {
        Some(body_schema) => {
            let expected_content_type = body_schema["content_type"].as_str().unwrap_or("");

            let actual_content_type = {
                let mut content_type = "";
                for header in request.headers().iter() {
                    let name = header.field.as_str().to_ascii_lowercase();

                    if name == "content-type" {
                        content_type = header.value.as_str();
                        break;
                    }
                }
                content_type
            };

            if !actual_content_type.starts_with(expected_content_type) {
                println!("Unauthorized request");
                return;
            }

            if !validate_body(body, body_schema) {
                println!("Unauthorized request");
                return;
            }
        }
        None => { // if there is no authorized body, but the request includes one
            if !body.is_empty() {
                println!("Unauthorized request");
                return;
            }
        }
    }

    forward_request(request, body, redirect_host, redirect_port);
}

// Parse a query string or urlencoded body into a HashMap
fn parse_query(query: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for (key, value) in form_urlencoded::parse(query.as_bytes()) {
        let key = key.into_owned();
        let value = value.into_owned();

        result.insert(key, value);
    }

    return result;
}

/// Check if a string follows the rules from the schema
fn validate_string(value: &str, schema: &Value) -> bool {

    // Check if an empty string is allowed
    let allow_empty = schema["allow_empty"].as_bool().unwrap_or(true);

    if value.is_empty() && !allow_empty {
        return false;
    }

    // Check the maximum length
    let max_length = schema["max_length"].as_u64();

    if max_length.is_some() {
        let max_length = max_length.unwrap();

        if value.len() as u64 > max_length {
            return false;
        }
    }

    // Check the allowed characters
    let allowed_chars = schema["allowed_chars"].as_str();

    if allowed_chars.is_some() {
        let allowed_chars = allowed_chars.unwrap();

        for character in value.chars() {
            if !allowed_chars.contains(character) {
                return false;
            }
        }
    }

    return true;
}

// Check if all query parameters are allowed
fn validate_query(query: &str, query_schema: &Value) -> bool {
    let parameters = parse_query(query);

    for (key, value) in &parameters {

        // Find the rules for this parameter
        let schema = query_schema.get(key.as_str());

        if schema.is_none() {
            // This parameter is not allowed
            return false;
        }

        let schema = schema.unwrap();

        // Check if the value follows the rules
        let is_valid = validate_string(value, schema);

        if !is_valid {
            return false;
        }
    }

    return true;
}

// Check if the request body follows the configured rules
fn validate_body(body: &str, body_schema: &Value) -> bool {

    // Get the expected content type
    let content_type = body_schema["content_type"]
        .as_str()
        .unwrap_or("");

    // Handle form-urlencoded data
    if content_type == "application/x-www-form-urlencoded" {

        let parameters = parse_query(body);
        let parameters_schema = &body_schema["parameters"];

        for (key, value) in &parameters {

            // Find the rules for this parameter
            let schema = parameters_schema.get(key.as_str());

            if schema.is_none() {
                return false;
            }

            let schema = schema.unwrap();

            let is_valid = validate_string(value, schema);

            if !is_valid {
                return false;
            }
        }

        return true;
    }

    // Handle JSON data
    if content_type == "application/json" {

        // Try to parse the body as JSON
        let json = serde_json::from_str::<JsonValue>(body);

        if json.is_err() {
            return false;
        }

        let json = json.unwrap();
        let parameters_schema = &body_schema["parameters"];

        // The JSON must contain an object
        let object = json.as_object();

        if object.is_none() {
            return false;
        }

        let object = object.unwrap();

        for (key, value) in object {

            // Find the rules for this parameter
            let schema = parameters_schema.get(key.as_str());

            if schema.is_none() {
                return false;
            }

            let schema = schema.unwrap();

            // Only strings are accepted
            let string_value = value.as_str();

            if string_value.is_none() {
                return false;
            }

            let string_value = string_value.unwrap();

            let is_valid = validate_string(string_value, schema);

            if !is_valid {
                return false;
            }
        }

        // Declared but absent JSON body params ARE enforced
        if let Some(mapping) = parameters_schema.as_mapping() {
            for (key, _) in mapping {
                let key = key.as_str().unwrap();
                if !object.contains_key(key) {
                    return false;
                }
            }
        }

        return true;
    }

    return false;
}

fn forward_request(request: Request, body: &str, redirect_host: &str, redirect_port: u16) {
    let server_url = format!("http://{}:{}", redirect_host, redirect_port);
    let url = format!("{}{}", server_url, request.url());

    let method = request.method().as_str();

    // Retrieves the original Content-Type to propagate it
    let content_type = request
        .headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case("content-type"))
        .map(|h| h.value.as_str().to_string());

    let mut req = ureq::request(method, &url);
    if let Some(ct) = content_type {
        req = req.set("Content-Type", &ct);
    }

    let response = match req.send_string(body) {
        Ok(r) => r,
        Err(ureq::Error::Status(_, r)) => r,
        Err(e) => {
            eprintln!("Backend unreachable: {} ({})", url, e);
            let resp = tiny_http::Response::from_string("Bad Gateway")
                .with_status_code(502);
            request.respond(resp).unwrap();
            return;
        }
    };

    let status = response.status();
    let mut resp_body = Vec::new();
    response.into_reader().read_to_end(&mut resp_body).unwrap();

    let response = tiny_http::Response::from_data(resp_body)
        .with_status_code(status);

    request.respond(response).unwrap();
}