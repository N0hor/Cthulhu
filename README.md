# Cthulhu
Most vulnerabilities come from characters you don't need. Cthulhu is a minimalist Web Application Firewall developed in Rust that validates what your application receives before the requests even reach it.

## Demo
```bash
docker compose build --no-cache
docker compose up
```

You can test the configuration like this:
```bash
bash proxy_test.sh
```

## Quick start
#### Pull docker images :
```bash
docker pull ghcr.io/n0hor/cthulhu:latest
```

#### Create docker-compose.yml :
```yml
services:
  cthulhu:
    image: ghcr.io/n0hor/cthulhu:latest
    ports:
      - "8080:8080"
    volumes:
      - ./conf.yml:/app/conf.yml:ro
    depends_on:
      - YOUR_APP

  YOUR_APP:
    build:
      context: .
      dockerfile: Dockerfile
    expose:
      - "7000"
```

#### Create conf.yml for the application

Example :
```yml
entry_port: 8080
port_to_redirect_to_after_verification: 7000
host_to_redirect_to_after_verification: "YOUR_APP"

paths:
  "/":
    GET:
      query:
        a:
          allowed_chars: "0123456789"
          max_length: 4
          allow_empty: false
        b:
          allowed_chars: "0123456789"
          max_length: 4
          allow_empty: false
    PUT:
      query:
        c:
          allowed_chars: "abcdefgh"
          max_length: 10
          allow_empty: false

  "/send_msg_form":
    POST:
      body:
        content_type: "application/x-www-form-urlencoded"
        parameters:
          msg:
            allowed_chars: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-"
            max_length: 50
            allow_empty: false

  "/send_msg_json":
    POST:
      body:
        content_type: "application/json"
        parameters:
          msg:
            allowed_chars: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-"
            max_length: 50
            allow_empty: false

  "/send_msg_with_query":
    POST:
      query:
        z:
          allowed_chars: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
          max_length: 20
          allow_empty: false
      body:
        content_type: "application/x-www-form-urlencoded"
        parameters:
          msg:
            allowed_chars: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-"
            max_length: 50
            allow_empty: false
```

#### Run
```bash
docker compose build --no-cache
docker compose up
```

## Programming paradigm
AI-assisted pseudocode driven development. Commented and adapted by a human.

## Bug bounty
I'll buy a beer to anyone who finds a vulnerability.
