use json::{self, JsonValue};
use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use threadpool::ThreadPool;
fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
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

    let mut response = Response::new();

    if request.req_line == "GET / HTTP/1.1" {
        response.get(
            "HTTP/1.1 200 OK".to_string(),
            fs::read_to_string("usr/test.html").unwrap(),
        );
    } else if request.req_line == "GET /snowday HTTP/1.1" {
        response.get(
            "HTTP/1.1 200 OK".to_string(),
            fs::read_to_string("usr/snowday.html").unwrap(),
        );
    } else if request.req_line == "GET /snowday.js HTTP/1.1" {
        response.get(
            "HTTP/1.1 200 OK".to_string(),
            fs::read_to_string("usr/snowday.js").unwrap(),
        );
    } else if request.req_line == "POST /snowday HTTP/1.1" {
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
        response.len = response.contents.to_string().len().to_string();
    }

    response.send(&stream);
}

struct Request {
    headers: Vec<String>,
    body: JsonValue,
    req_line: String
}
struct Response {
    status: String,
    contents: Content,
    len: String,
}
enum Content {
    StringContent(String),
    VecContent(Vec<Vec<String>>),
}
impl Content {
    fn to_string(&self) -> String {
        match self {
            Content::StringContent(s) => s.clone(),
            Content::VecContent(v) => {
                let mut ans = String::new();
                for i in v {
                    ans.push_str(&format!("{:?}", i));
                }
                return ans;
            }
        }
    }
}
impl Response {
    fn new() -> Response {
        return Response {
            status: "HTTP/1.1 404 NOT FOUND".to_string(),
            contents: Content::StringContent(fs::read_to_string("usr/404.html").unwrap()),
            len: fs::read_to_string("usr/404.html")
                .unwrap()
                .len()
                .to_string(),
        };
    }
    fn get(&mut self, status: String, contents: String) {
        self.status = status;
        self.contents = Content::StringContent(contents.clone());
        self.len = contents.len().to_string();
    }
    fn send(&self, mut stream: &TcpStream) {
        stream
            .write_all(
                format!(
                    "{status}\r\nContent-Length: {len}\r\n\r\n{contents}",
                    status = self.status,
                    len = self.len,
                    contents = self.contents.to_string()
                )
                .as_bytes(),
            )
            .unwrap();
    }
}

impl Request {
    fn new(stream: TcpStream) -> Request {
        let mut buf_reader = BufReader::new(&stream);
        let mut headers = Vec::new();
        let mut body = JsonValue::new_object();
        loop {
            let mut line = String::new();
            let bytes_read = buf_reader.read_line(&mut line).unwrap();

            if bytes_read == 0 {
                break;
            }

            if line.trim().is_empty() {
                break;
            }

            headers.push(line);
        }

        let req_line = headers[0].clone().replace("\r\n", "");
        if req_line.starts_with("POST") {
            let mut request_body = vec![
                0;
                headers
                    .iter()
                    .find(|h| h.starts_with("Content-Length:"))
                    .unwrap()
                    .split(":")
                    .last()
                    .unwrap()
                    .trim()
                    .parse::<usize>()
                    .unwrap()
            ];
            let _ = buf_reader.read_exact(&mut request_body).unwrap();
            let new = String::from_utf8(request_body).unwrap();
            body = json::parse(&new).unwrap();
        }
        return Request {
            headers,
            body,
            req_line
        };
    }
}

async fn get_zip_code(zipcode: usize) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let url = format!(
        "http://api.geonames.org/postalCodeLookupJSON?postalcode={}&username=aaaa",
        zipcode
    );
    let response = reqwest::get(&url).await?.text().await?;
    let parsed = json::parse(&response).unwrap();
    let ans: Vec<Vec<String>> = parsed["postalcodes"]
        .members()
        .map(|x| {
            vec![
                x["placeName"].to_string(),
                x["lat"].to_string(),
                x["lng"].to_string(),
            ]
        })
        .collect();
    return Ok(ans);
}
