use std::time::Duration;

const HOST: &str = "http://localhost:8080";
const BOUNDARY: &str = "----CthulhuTestBoundary";

struct Case {
    name: &'static str,
    method: &'static str,
    path: &'static str,
    content_type: Option<String>,
    body: Vec<u8>,
    expected: u16,
}

fn case(
    name: &'static str,
    method: &'static str,
    path: &'static str,
    content_type: Option<&str>,
    body: Vec<u8>,
    expected: u16,
) -> Case {
    Case {
        name,
        method,
        path,
        content_type: content_type.map(|s| s.to_string()),
        body,
        expected,
    }
}

fn http(
    method: &str,
    path: &str,
    content_type: Option<&str>,
    body: &[u8],
) -> (u16, String) {
    let url = format!("{}{}", HOST, path);
    let mut req = ureq::request(method, &url).timeout(Duration::from_secs(5));
    if let Some(ct) = content_type {
        req = req.set("Content-Type", ct);
    }
    let result = if method == "GET" {
        req.call()
    } else {
        req.send_bytes(body)
    };
    match result {
        Ok(r) => (r.status(), r.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(code, r)) => (code, r.into_string().unwrap_or_default()),
        Err(e) => panic!("transport error on {} {}: {}", method, url, e),
    }
}

fn multipart(filename: &str, data: &[u8]) -> (String, Vec<u8>) {
    let ct = format!("multipart/form-data; boundary={}", BOUNDARY);
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", BOUNDARY).as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n",
            filename
        )
        .as_bytes(),
    );
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{}--\r\n", BOUNDARY).as_bytes());
    (ct, body)
}

fn png_bytes() -> Vec<u8> {
    vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0]
}
fn jpg_bytes() -> Vec<u8> {
    vec![0xFF, 0xD8, 0xFF, 0xE0, 0, 0, 0, 0]
}
fn gif_bytes() -> Vec<u8> {
    let mut v = b"GIF89a".to_vec();
    v.extend_from_slice(&[0, 0, 0, 0]);
    v
}


#[test]
fn proxy_end_to_end() {
    let mut cases: Vec<Case> = Vec::new();

    cases.push(case("T01 GET / (no params)", "GET", "/", None, vec![], 200));
    cases.push(case("T02 GET /?a=1234&b=5678", "GET", "/?a=1234&b=5678", None, vec![], 200));
    cases.push(case("T03 GET /unknown", "GET", "/unknown", None, vec![], 403));
    cases.push(case("T04 POST / (method not allowed)", "POST", "/", None, vec![], 403));
    cases.push(case("T05 GET /send_msg_form (method not allowed)", "GET", "/send_msg_form", None, vec![], 403));
    cases.push(case("T06 GET /send_msg_form/ (trailing slash)", "GET", "/send_msg_form/", None, vec![], 403));
    cases.push(case("T07 GET /Send_Msg_Form (case sensitive)", "GET", "/Send_Msg_Form", None, vec![], 403));

    cases.push(case("T08 GET /?a=9999 (upper limit)", "GET", "/?a=9999", None, vec![], 200));
    cases.push(case("T09 GET /?a=12345 (too long)", "GET", "/?a=12345", None, vec![], 403));
    cases.push(case("T10 GET /?a=12a3 (forbidden char)", "GET", "/?a=12a3", None, vec![], 403));
    cases.push(case("T11 GET /?a= (empty forbidden)", "GET", "/?a=", None, vec![], 403));
    cases.push(case("T12 GET /?a=-1 (minus forbidden)", "GET", "/?a=-1", None, vec![], 403));
    cases.push(case("T13 GET /?a=%21 (encoded !)", "GET", "/?a=%21", None, vec![], 403));
    cases.push(case("T14 GET /?a=12&a=99 (dup key)", "GET", "/?a=12&a=99", None, vec![], 403));
    cases.push(case("T15 GET /?c=abc (c not allowed)", "GET", "/?c=abc", None, vec![], 403));
    cases.push(case("T16 GET /?b=42", "GET", "/?b=42", None, vec![], 200));
    cases.push(case("T17 PUT /?c=abcdefgh (valid)", "PUT", "/?c=abcdefgh", None, vec![], 200));
    cases.push(case("T18 PUT /?c=ABCDEFGH", "PUT", "/?c=ABCDEFGH", None, vec![], 403));
    cases.push(case("T19 PUT /?c=abcdefghijk (too long)", "PUT", "/?c=abcdefghijk", None, vec![], 403));
    cases.push(case("T20 PUT /?c=abc123", "PUT", "/?c=abc123", None, vec![], 403));

    let form = Some("application/x-www-form-urlencoded");
    cases.push(case("T21 POST form msg=hello", "POST", "/send_msg_form", form, b"msg=hello".to_vec(), 200));
    cases.push(case("T22 POST form msg=hello_world-42", "POST", "/send_msg_form", form, b"msg=hello_world-42".to_vec(), 200));
    cases.push(case("T23 POST form msg=paff!!!! (forbidden char)", "POST", "/send_msg_form", form, b"msg=paff%21%21%21%21".to_vec(), 403));
    cases.push(case("T24 POST form msg= (empty)", "POST", "/send_msg_form", form, b"msg=".to_vec(), 403));
    cases.push(case("T25 POST form msg=hello&other=x", "POST", "/send_msg_form", form, b"msg=hello&other=x".to_vec(), 403));
    cases.push(case("T26 POST form (no body, no CT)", "POST", "/send_msg_form", None, vec![], 403));
    cases.push(case("T27 POST form msg=<script> raw", "POST", "/send_msg_form", form, b"msg=%3Cscript%3E".to_vec(), 403));
    cases.push(case("T28 POST form msg=%3Cscript%3E", "POST", "/send_msg_form", form, b"msg=%3Cscript%3E".to_vec(), 403));
    cases.push(case("T29 POST form CT=text/plain", "POST", "/send_msg_form", Some("text/plain"), b"msg=hello".to_vec(), 403));
    cases.push(case("T30 POST form CT=application/json", "POST", "/send_msg_form", Some("application/json"), b"msg=hello".to_vec(), 403));
    cases.push(case("T31 POST form CT+charset", "POST", "/send_msg_form", Some("application/x-www-form-urlencoded; charset=UTF-8"), b"msg=hello".to_vec(), 200));
    cases.push(case("T32 POST form msg too long (60 a)", "POST", "/send_msg_form", form, format!("msg={}", "a".repeat(60)).into_bytes(), 403));
    cases.push(case("T45 POST form msg=hello%00world", "POST", "/send_msg_form", form, b"msg=hello%00world".to_vec(), 403));
    cases.push(case("T46 POST form msg=%E2%82%AC (euro)", "POST", "/send_msg_form", form, b"msg=%E2%82%AC".to_vec(), 403));
    cases.push(case("T47 POST form + stray query", "POST", "/send_msg_form?foo=bar", form, b"msg=hello".to_vec(), 403));

    let json = Some("application/json");
    cases.push(case("T33 POST JSON msg=yolo", "POST", "/send_msg_json", json, b"{\"msg\":\"yolo\"}".to_vec(), 200));
    cases.push(case("T34 POST JSON msg=paff!!!!", "POST", "/send_msg_json", json, b"{\"msg\":\"paff!!!!\"}".to_vec(), 403));
    cases.push(case("T35 POST JSON invalid", "POST", "/send_msg_json", json, b"{msg:\"yolo\"}".to_vec(), 403));
    cases.push(case("T36 POST JSON msg=42 (not string)", "POST", "/send_msg_json", json, b"{\"msg\":42}".to_vec(), 403));
    cases.push(case("T37 POST JSON extra param", "POST", "/send_msg_json", json, b"{\"msg\":\"yolo\",\"extra\":\"x\"}".to_vec(), 403));
    cases.push(case("T38 POST JSON {} (missing msg)", "POST", "/send_msg_json", json, b"{}".to_vec(), 403));
    cases.push(case("T39 POST JSON msg empty", "POST", "/send_msg_json", json, b"{\"msg\":\"\"}".to_vec(), 403));
    cases.push(case("T40 POST JSON []", "POST", "/send_msg_json", json, b"[]".to_vec(), 403));
    cases.push(case("T41 POST JSON null", "POST", "/send_msg_json", json, b"null".to_vec(), 403));
    cases.push(case("T42 POST JSON empty body", "POST", "/send_msg_json", json, vec![], 403));
    cases.push(case("T43 POST JSON CT+charset", "POST", "/send_msg_json", Some("application/json; charset=utf-8"), b"{\"msg\":\"yolo\"}".to_vec(), 200));
    cases.push(case("T44 POST JSON ünicode", "POST", "/send_msg_json", json, b"{\"msg\":\"\xc3\xbcnicode\"}".to_vec(), 403));
    cases.push(case("T48 POST JSON + stray query", "POST", "/send_msg_json?foo=bar", json, b"{\"msg\":\"yolo\"}".to_vec(), 403));

    cases.push(case("T49 GET /?a=0000", "GET", "/?a=0000", None, vec![], 200));
    cases.push(case("T50 GET /?a=%30%31%32%33", "GET", "/?a=%30%31%32%33", None, vec![], 200));

    cases.push(case("T51 POST wq valid", "POST", "/send_msg_with_query?z=abc123", form, b"msg=hello".to_vec(), 200));
    cases.push(case("T52 POST wq msg2 not allowed", "POST", "/send_msg_with_query?z=abc123", form, b"msg=hello&msg2=x".to_vec(), 403));
    cases.push(case("T53 POST wq z empty", "POST", "/send_msg_with_query?z=", form, b"msg=hello".to_vec(), 403));
    cases.push(case("T54 POST wq z=ABC", "POST", "/send_msg_with_query?z=ABC", form, b"msg=hello".to_vec(), 200));
    cases.push(case("T55 POST wq z too long", "POST", "/send_msg_with_query?z=abcdefghijklmnopqrstuvw", form, b"msg=hello".to_vec(), 403));
    cases.push(case("T56 POST wq z has dash", "POST", "/send_msg_with_query?z=abc-123", form, b"msg=hello".to_vec(), 403));
    cases.push(case("T57 POST wq missing z", "POST", "/send_msg_with_query", form, b"msg=hello".to_vec(), 200));
    cases.push(case("T58 POST wq msg bad", "POST", "/send_msg_with_query?z=abc123", form, b"msg=paff!!!!".to_vec(), 403));
    cases.push(case("T59 POST wq extra query param", "POST", "/send_msg_with_query?z=abc123&other=x", form, b"msg=hello".to_vec(), 403));
    cases.push(case("T60 POST wq CT mismatch", "POST", "/send_msg_with_query?z=abc123", json, b"{\"msg\":\"hello\"}".to_vec(), 403));

    {
        let (ct, body) = multipart("test.png", &png_bytes());
        cases.push(case("IMG1 valid PNG", "POST", "/upload_image", Some(&ct), body, 200));
    }
    {
        let (ct, body) = multipart("test.jpg", &jpg_bytes());
        cases.push(case("IMG2 valid JPEG", "POST", "/upload_image", Some(&ct), body, 200));
    }
    {
        let (ct, body) = multipart("test.gif", &gif_bytes());
        cases.push(case("IMG3 valid GIF", "POST", "/upload_image", Some(&ct), body, 200));
    }
    {
        let (ct, body) = multipart("test.txt", &png_bytes());
        cases.push(case("IMG4 bad extension (.txt)", "POST", "/upload_image", Some(&ct), body, 403));
    }
    {
        let (ct, body) = multipart("test.exe", &png_bytes());
        cases.push(case("IMG5 forbidden extension (.exe)", "POST", "/upload_image", Some(&ct), body, 403));
    }
    {
        let (ct, body) = multipart("test.png", b"not an image");
        cases.push(case("IMG6 wrong magic", "POST", "/upload_image", Some(&ct), body, 403));
    }
    {
        let (ct, body) = multipart("test.jpg", &png_bytes());
        cases.push(case("IMG7 extension/magic mismatch", "POST", "/upload_image", Some(&ct), body, 403));
    }

    let mut failures: Vec<String> = Vec::new();

    for c in &cases {
        let (status, _body) = http(c.method, c.path, c.content_type.as_deref(), &c.body);
        let ok = status == c.expected;
        let marker = if ok { "PASS" } else { "FAIL" };
        println!(
            "[{}] {} -> {} (expected {})",
            marker, c.name, status, c.expected
        );
        if !ok {
            failures.push(format!(
                "{}: got {}, expected {}",
                c.name, status, c.expected
            ));
        }
    }

    println!(
        "\n{}/{} tests passed",
        cases.len() - failures.len(),
        cases.len()
    );

    if !failures.is_empty() {
        panic!("Failures:\n  - {}", failures.join("\n  - "));
    }
}