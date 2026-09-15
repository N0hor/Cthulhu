// AI-assisted pseudocode driven development
// Commented and adapted by a human

use serde_yaml::{from_reader, Value};
use std::fs::File;
use std::io::Read;
use tiny_http::{Server, Request};
use std::collections::{HashMap, HashSet};
use serde_json::Value as JsonValue;
use url::form_urlencoded;
use multipart::server::Multipart;
use std::io::Cursor;

const CONFIG_PATH: &str = "conf.yml";
const MAX_BODY: u64 = 10 * 1024 * 1024;

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
            let mut bytes = Vec::new();

            // Reads up to MAX_BODY + 1 byte to detect an overflow.
            request.as_reader()
                .take(MAX_BODY + 1)
                .read_to_end(&mut bytes)
                .unwrap();

            if bytes.len() as u64 > MAX_BODY {
                return_an_invalid_request_to_client(request);
                continue;
            }

            bytes
        };

        handle_request(request, &body, paths, redirect_host, redirect_port);
    }
}

fn handle_request(request: Request, body: &[u8], paths: &Value, redirect_host: &str, redirect_port: u16) {

    // Get method
    let method = request.method().to_string();

    // Parse url : path + query
    let full_url = request.url().to_string();
    let (requested_path, query_string) = match full_url.split_once('?') {
        Some((path, query)) => (path.to_string(), query.to_string()),
        None => (full_url.clone(), String::new()),
    };

    // Is requested_path match a path in the config ?
    let path_config = match paths.get(&requested_path) {
        Some(value) => value, // if we match value yes !
        None => { return_an_invalid_request_to_client(request); return; } // else
    };

    // Is method allowed for this path in the config ?
    let method_config = match path_config.get(&method) {
        Some(value) => value,
        None => { return_an_invalid_request_to_client(request); return; }
    };

    // Query string validation
    match method_config.get("query") {
        Some(query_schema) => { // if there is a query parameter
            if !validate_query(&query_string, query_schema) { // but not allowed in the conf
                return_an_invalid_request_to_client(request);
                return;
            }
        }
        None => { // if their is a query parameter, but without content 
            if !query_string.is_empty() {
                return_an_invalid_request_to_client(request);
                return;
            }
        }
    }

    // Body validation
    match method_config.get("body") {
        Some(body_schema) => {
            let expected_content_type = body_schema["content_type"].as_str().unwrap_or("");

            let actual_content_type: String = request
                .headers()
                .iter()
                .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case("content-type"))
                .map(|h| h.value.as_str().to_string())
                .unwrap_or_default();

            // Mime check skipped for images
            if expected_content_type != "image" {
                let actual_mime = actual_content_type
                    .split(';').next().unwrap_or("").trim().to_ascii_lowercase();
                let expected_mime = expected_content_type
                    .split(';').next().unwrap_or("").trim().to_ascii_lowercase();

                if actual_mime != expected_mime {
                    return_an_invalid_request_to_client(request);
                    return;
                }
            }

            if !validate_body(body, body_schema, &actual_content_type) {
                return_an_invalid_request_to_client(request);
                return;
            }
        }
        None => {
            if !body.is_empty() {
                return_an_invalid_request_to_client(request);
                return;
            }
        }
    }

    forward_request(request, body, &requested_path, &query_string, redirect_host, redirect_port);
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

    // Duplicate query keys ARE detected and forbidden (see 0.0.4 version)
    let mut seen = HashSet::new();

    for (key, _) in form_urlencoded::parse(query.as_bytes()) {
        if !seen.insert(key.into_owned()) {
            return false;
        }
    }


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
fn validate_body(body: &[u8], body_schema: &Value, actual_content_type: &str) -> bool {

    // Get the expected content type
    let content_type = body_schema["content_type"]
        .as_str()
        .unwrap_or("");

    if content_type == "image" {
        return validate_image(body, body_schema, actual_content_type);
    }

    let body_str = match std::str::from_utf8(body) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Handle form-urlencoded data
    if content_type == "application/x-www-form-urlencoded" {

        // Detect content type duplication
        let mut seen = HashSet::new();
        for (key, _) in form_urlencoded::parse(body_str.as_bytes()) {
            if !seen.insert(key.into_owned()) {
                return false;
            }
        }

        let parameters = parse_query(body_str);
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

        // Declared but absent form-urlencoded params ARE enforced
        if let Some(mapping) = parameters_schema.as_mapping() {
            for (key, _) in mapping {
                let key = key.as_str().unwrap();
                if !parameters.contains_key(key) {
                    return false;
                }
            }
        }

        return true;
    }

    // Handle JSON data
    if content_type == "application/json" {

        // Try to parse the body as JSON
        let json = serde_json::from_str::<JsonValue>(body_str);

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

fn forward_request(
    request: Request,
    body: &[u8],
    requested_path: &str,
    query_string: &str,
    redirect_host: &str,
    redirect_port: u16,
) {
    // Constructs a safe base URL
    let mut url = match url::Url::parse(&format!("http://{}:{}", redirect_host, redirect_port)) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Invalid backend URL: {}", e);
            let resp = tiny_http::Response::from_string("Bad Gateway")
                .with_status_code(502);
            request.respond(resp).unwrap();
            return;
        }
    };

    url.set_path(requested_path);
    url.set_query(if query_string.is_empty() {
        None
    } else {
        Some(query_string)
    });

    let method = request.method().as_str();

    // Retrieves the original Content-Type to propagate it
    let content_type = request
        .headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case("content-type"))
        .map(|h| h.value.as_str().to_string());

    let mut req = ureq::request(method, url.as_str());
    if let Some(ct) = content_type {
        req = req.set("Content-Type", &ct);
    }

    let response = match req.send_bytes(body) {
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
    response.into_reader()
        .take(MAX_BODY + 1)
        .read_to_end(&mut resp_body)
        .unwrap();

    if resp_body.len() as u64 > MAX_BODY {
        eprintln!("Backend response too large");
        let resp = tiny_http::Response::from_string("Bad Gateway")
            .with_status_code(502);
        request.respond(resp).unwrap();
        return;
    }

    let response = tiny_http::Response::from_data(resp_body)
        .with_status_code(status);

    request.respond(response).unwrap();
}

fn return_an_invalid_request_to_client(request: Request) {
    let resp = tiny_http::Response::from_string("Forbidden")
        .with_status_code(403);
    request.respond(resp).unwrap();
}

fn validate_image(body: &[u8], body_schema: &Value, actual_content_type: &str) -> bool {
    // Boundary issu of Content-Type
    let boundary = match actual_content_type.split(';')
        .find_map(|p| p.trim().strip_prefix("boundary="))
    {
        Some(b) => b.trim_matches('"').to_string(),
        None => return false,
    };

    let max_size = body_schema["max_size"].as_u64().unwrap_or(u64::MAX);

    let allowed_extensions: Vec<String> = match body_schema["allowed_extensions"].as_sequence() {
        Some(seq) => seq.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_ascii_lowercase()))
            .collect(),
        None => return false,
    };

    let mut multipart = Multipart::with_body(Cursor::new(body), boundary);

    let mut found = false;

    while let Ok(Some(mut field)) = multipart.read_entry() {
        let filename = match &field.headers.filename {
            Some(f) => f.clone(),
            None => continue,
        };

        // File extension validation
        let file_ext = match std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| normalize_ext(&e.to_ascii_lowercase()))
        {
            Some(e) => e,
            None => return false,
        };

        if !allowed_extensions.iter().any(|e| normalize_ext(e) == file_ext) {
            return false;
        }

        let mut data = Vec::new();
        if field.data.read_to_end(&mut data).is_err() {
            return false;
        }
        if data.len() as u64 > max_size {
            return false;
        }

        let kind = match infer::get(&data) {
            Some(k) => k,
            None => return false,
        };

        if !kind.mime_type().starts_with("image/") {
            return false;
        }

        // Consistency between file extension and detected type
        if normalize_ext(&kind.extension().to_ascii_lowercase()) != file_ext {
            return false;
        }

        found = true;
        break;
    }

    found
}

fn normalize_ext(ext: &str) -> String {
    match ext {
        "jpeg" => "jpg".to_string(),
        other => other.to_string(),
    }
}