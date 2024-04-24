use json::{self, JsonValue};
use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;
use std::path::Path;
mod main;
pub struct Request {
    pub headers: Vec<String>,
    pub body: JsonValue,
    pub req_line: String,
}
pub struct Response {
    pub status: String,
    pub contents: Content,
    pub len: String,
    pub content_type: String,
}
#[derive(Debug)]
pub enum Content {
    StringContent(String),
    VecContent(Vec<Vec<String>>),
    FloatContent(f32),
}
impl Content {
    pub fn to_string(&self) -> String {
        match self {
            Content::StringContent(s) => s.clone(),
            Content::VecContent(v) => {
                let mut ans = String::new();
                for i in v {
                    ans.push_str(&format!("{:?}", i));
                }
                return ans;
            }
            Content::FloatContent(i) => i.to_string(),
        }
    }
    pub fn to_json(&self) -> JsonValue {
        match self {
            Content::StringContent(s) => json::parse(s).unwrap(),
            Content::VecContent(v) => {
                let mut ans = JsonValue::new_array();
                for i in v {
                    let mut temp = JsonValue::new_array();
                    for j in i {
                        temp.push(j.clone()).unwrap();
                    }
                    ans.push(temp).unwrap();
                }
                return ans;
            }
            Content::FloatContent(i) => {
                return json::parse(&format!("{}", i)).unwrap();
            }
        }
    }
    pub fn round(&self) -> Content {
        match self {
            Content::FloatContent(f) => {
                let mut rounded: f32 = f * 1000f32;
                rounded = rounded.floor();
                rounded = rounded / 1000f32;
                return Content::FloatContent(rounded);
            }
            _ => Content::FloatContent(0f32),
        }
    }
}

impl Response {
    pub fn new() -> Response {
        return Response {
            status: "HTTP/1.1 404 NOT FOUND".to_string(),
            contents: Content::StringContent(fs::read_to_string("usr/404.html").unwrap()),
            len: fs::read_to_string("usr/404.html")
                .unwrap()
                .len()
                .to_string(),
            content_type: "text/html".to_string(),
        };
    }
    pub fn get(&mut self, status: String, contents: String, content_type: String) {
        self.status = status;
        self.contents = Content::StringContent(contents.clone());
        self.content_type = content_type;
        self.len = contents.len().to_string();
    }
    pub fn put(
        &mut self,
        status: String,
        contents: String,
        content_type: String,
        file_path: String,
        content_for_file: String,
    ) {
        fs::write(file_path, content_for_file).unwrap();
        self.status = status;
        self.contents = Content::StringContent(contents.clone());
        self.content_type = content_type;
        self.len = contents.len().to_string();
    }
    pub fn head(&mut self, status: String, contents: String, content_type: String) {
        self.status = status;
        self.contents = Content::StringContent("".to_string());
        self.content_type = content_type;
        self.len = contents.len().to_string();
    }
    pub fn delete(&mut self, status: String, file_path: String) {
        if Path::new(&file_path).exists() {
            fs::remove_file(file_path).unwrap();
            self.status = status;
            self.contents = Content::StringContent("".to_string());
            self.content_type = "text/html".to_string();
            self.len = "".len().to_string();
        } else {
            self.status = "HTTP/1.1 410 GONE".to_string();
            self.contents =
                Content::StringContent("410 ".to_owned() + file_path.as_str() + " not found!");
            self.len = self.contents.to_string().len().to_string();
            self.content_type = "text/html".to_string();
        }
    }
    pub fn options(&mut self, path: String, header: &String) {
        if header == "WARNING: THIS IS A TEST" {
            self.status = "HTTP/1.1 204 No Content".to_string();
            return;
        }
        let options = vec!["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS"];
        let mut allowed: Vec<String> = vec![];
        for i in options {
            let a = Request {
                headers: vec!["WARNING: THIS IS A TEST".to_string()],
                body: JsonValue::Null,
                req_line: i.to_string() + " " + path.as_str(),
            };
            if process(a).status != "HTTP/1.1 404 NOT FOUND" {
                allowed.push(i.to_string());
            }
        }
        self.status = "HTTP/1.1 200 OK".to_string();
        self.contents = Content::StringContent(allowed.join(", "));
        self.len = self.contents.to_string().len().to_string();
        self.content_type = "text/html".to_string();
    }
    pub fn send(&mut self, mut stream: &TcpStream) {
        match self.contents {
            Content::StringContent(_) => {
                stream
                    .write_all(
                        format!(
                            "{status}\r\nContent-Length: {len}\r\nContent-Type: {content_type}\r\n\r\n{contents}",
                            status = self.status,
                            len = self.len,
                            content_type = self.content_type,
                            contents = self.contents.to_string()
                        )
                        .as_bytes(),
                    )
                    .unwrap();
            }
            Content::VecContent(_) => {
                self.len = self.contents.to_json().dump().len().to_string();
                stream
                    .write_all(
                        format!(
                            "{status}\r\nContent-Length: {len}\r\n\r\n{contents}",
                            status = self.status,
                            contents = self.contents.to_json().dump(),
                            len = self.len,
                        )
                        .as_bytes(),
                    )
                    .unwrap();
            }
            Content::FloatContent(_) => {
                println!(" HERE: {:?}", self.contents);
                self.contents = self.contents.round();
                self.len = self.contents.to_json().dump().len().to_string();
                stream
                    .write_all(
                        format!(
                            "{status}\r\nContent-Length: {len}\r\n\r\n{contents}",
                            status = self.status,
                            contents = self.contents.to_json().dump(),
                            len = self.len,
                        )
                        .as_bytes(),
                    )
                    .unwrap();
            }
        }
    }
}

impl Request {
    pub fn new(stream: TcpStream) -> Request {
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

        let req_line: String = headers[0]
            .clone()
            .replace("\r\n", "")
            .replace(" HTTP/1.1", "");
        if headers.iter().any(|h| h.contains("Content-Length:")) {
            // if req has a body, do this :3
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
            req_line,
        };
    }
}