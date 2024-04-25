use json::{self, JsonValue};
use std::fs;
use std::net::{TcpListener, TcpStream};
use threadpool::ThreadPool;

mod server_crate;
use server_crate::{Content, Request, Response};

mod snowdaycalc;
use snowdaycalc::{get_snow_day_chances, get_zip_code};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle(stream);
        });
    }
}

fn handle(stream: TcpStream) {
    let request = Request::new(stream.try_clone().unwrap());
    println!("Request line: {:?}", request.req_line);
    let mut response = process(request);
    response.send(&stream);
}

fn process(request: Request) -> Response {
    let mut response = Response::new();
    if request.req_line == "GET /" {
        response.get(
            "HTTP/1.1 200 OK".to_string(),
            fs::read_to_string("usr/test.html").unwrap(),
            "text/html".to_string(),
        );
    }
    if request.req_line == "GET /snowday" {
        response.get(
            "HTTP/1.1 200 OK".to_string(),
            fs::read_to_string("usr/snowday.html").unwrap(),
            "text/html".to_string(),
        );
    }
    if request.req_line == "GET /snowday.js" {
        response.get(
            "HTTP/1.1 200 OK".to_string(),
            fs::read_to_string("usr/snowday.js").unwrap(),
            "application/javascript".to_string(),
        );
    }
    if request.req_line == "POST /snowday" {
        if request.body == JsonValue::Null {
            response.get(
                "HTTP/1.1 400 BAD REQUEST".to_string(),
                "400 BAD REQUEST".to_string(),
                "text/html".to_string(),
            );
            return response;
        }
        response.status = "HTTP/1.1 200 OK".to_string().to_owned();
        let parsed = request.body.clone();
        let zipcode: usize = parsed["zipcode"].to_string().parse().unwrap();
        response.contents = Content::VecContent(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async { get_zip_code(zipcode).await.unwrap() }),
        );
        response.content_type = "application/json".to_string();
        response.len = response.contents.to_string().len().to_string();
    }
    if request.req_line == "POST /snowday_latlong" {
        if request.body == JsonValue::Null {
            response.get(
                "HTTP/1.1 400 BAD REQUEST".to_string(),
                "400 BAD REQUEST".to_string(),
                "text/html".to_string(),
            );
            return response;
        }
        response.status = "HTTP/1.1 200 OK".to_string().to_owned();
        let parsed = request.body.clone();
        let lat: f64 = parsed["lat"].to_string().parse().unwrap();
        let lng: f64 = parsed["lng"].to_string().parse().unwrap();
        println!("lat: {}, lng: {}", lat, lng);
        response.contents = Content::FloatContent(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async { get_snow_day_chances(lat as u32, lng as u32).await.unwrap() }),
        );
        response.content_type = "application/json".to_string();
    }
    if request.req_line == "GET /favicon.ico" {
        response.status = "HTTP/1.1 200 OK".to_string().to_owned();
    }
    if request.req_line == "PUT /waow.html" {
        if request
            .headers
            .contains(&"Password: transrights\r\n".to_string())
            && request.body != JsonValue::Null
        {
            response.put(
                "HTTP/1.1 200 OK".to_string(),
                "OK".to_string(),
                "text/html".to_string(),
                "file/waow.html".to_string(),
                request.body["contents"].to_string(),
            );
            response.status = "HTTP/1.1 200 OK".to_string().to_owned();
        } else {
            response.get(
                "HTTP/1.1 401 UNAUTHORIZED".to_string(),
                "401 UNAUTHORIZED".to_string(),
                "text/html".to_string(),
            );
        }
    }
    if request.req_line == "GET /test.js" {
        response.get(
            "HTTP/1.1 200 OK".to_string(),
            fs::read_to_string("usr/test.js").unwrap(),
            "application/javascript".to_string(),
        );
    }
    if request.req_line == "HEAD /snowday" {
        response.head(
            "HTTP/1.1 200 OK".to_string(),
            fs::read_to_string("usr/snowday.html").unwrap(),
            "text/html".to_string(),
        );
    }
    if request.req_line == "DELETE /waow.html" {
        if request
            .headers
            .contains(&"Password: transrights\r\n".to_string())
        {
            response.delete("HTTP/1.1 200 OK".to_string(), "file/waow.html".to_string());
        } else {
            response.get(
                "HTTP/1.1 401 UNAUTHORIZED".to_string(),
                "401 UNAUTHORIZED".to_string(),
                "text/html".to_string(),
            );
        }
    }
    if request.req_line == "OPTIONS /snowday" {
        response.options("/snowday".to_string(), &request.headers[0], process);
        
    }
    return response;
}
