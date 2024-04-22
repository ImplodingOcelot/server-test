use json::{self, JsonValue};
use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use threadpool::ThreadPool;
use std::path::Path;
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
    let mut response = Response::new();

    if request.req_line == "GET /" {
        response.get(
            "HTTP/1.1 200 OK".to_string(),
            fs::read_to_string("usr/test.html").unwrap(),
            "text/html".to_string(),
        );
    } else if request.req_line == "GET /snowday" {
        response.get(
            "HTTP/1.1 200 OK".to_string(),
            fs::read_to_string("usr/snowday.html").unwrap(),
            "text/html".to_string(),
        );
    } else if request.req_line == "GET /snowday.js" {
        response.get(
            "HTTP/1.1 200 OK".to_string(),
            fs::read_to_string("usr/snowday.js").unwrap(),
            "application/javascript".to_string(),
        );
    } else if request.req_line == "POST /snowday" {
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
    } else if request.req_line == "POST /snowday_latlong" {
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
    } else if request.req_line == "GET /favicon.ico" {
        response.status = "HTTP/1.1 200 OK".to_string().to_owned();
    } else if request.req_line == "PUT /waow.html" {
        if request
            .headers
            .contains(&"Password: transrights\r\n".to_string())
            && request.body != JsonValue::Null
        {
            // fs::write("file/waow.html", request.body["contents"].to_string()).unwrap();
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
    } else if request.req_line == "GET /test.js" {
        response.get(
            "HTTP/1.1 200 OK".to_string(),
            fs::read_to_string("usr/test.js").unwrap(),
            "application/javascript".to_string(),
        );
    } else if request.req_line == "HEAD /snowday" {
        response.head(
            "HTTP/1.1 200 OK".to_string(),
            fs::read_to_string("usr/snowday.html").unwrap(),
            "text/html".to_string(),
        );
    } else if request.req_line == "DELETE /waow.html" {
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

    response.send(&stream);
}

struct Request {
    headers: Vec<String>,
    body: JsonValue,
    req_line: String,
}
struct Response {
    status: String,
    contents: Content,
    len: String,
    content_type: String,
}
#[derive(Debug)]
enum Content {
    StringContent(String),
    VecContent(Vec<Vec<String>>),
    FloatContent(f32),
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
            Content::FloatContent(i) => i.to_string(),
        }
    }
    fn to_json(&self) -> JsonValue {
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
    fn round(&self) -> Content {
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
    fn new() -> Response {
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
    fn get(&mut self, status: String, contents: String, content_type: String) {
        self.status = status;
        self.contents = Content::StringContent(contents.clone());
        self.content_type = content_type;
        self.len = contents.len().to_string();
    }
    fn put(
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
    fn head(&mut self, status: String, contents: String, content_type: String) {
        self.status = status;
        self.contents = Content::StringContent("".to_string());
        self.content_type = content_type;
        self.len = contents.len().to_string();
    }
    fn delete(&mut self, status: String, file_path: String) {
        if Path::new(&file_path).exists() {
            fs::remove_file(file_path).unwrap();
            self.status = status;
            self.contents = Content::StringContent("".to_string());
            self.content_type = "text/html".to_string();
            self.len = "".len().to_string();
        } else {
            self.status = "HTTP/1.1 404 NOT FOUND".to_string();
            self.contents = Content::StringContent("404 ".to_owned() + file_path.as_str() + " not found!");
            self.len = self.contents.to_string().len().to_string();
            self.content_type = "text/html".to_string();
        }
    }
    fn send(&mut self, mut stream: &TcpStream) {
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

async fn get_snow_day_chances(lat: u32, lng: u32) -> Result<f32, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m,apparent_temperature,precipitation_probability,precipitation,snowfall,visibility,wind_speed_10m,&forecast_days=2",
        lat, lng
    );
    /*
    Parematers:
    hourly_temp
    relative_humidity
    dew_point
    apparent_temperature
    precipitation_probability
    precipitation
    snowfall
    visibility
    wind_speed
    */
    let response = reqwest::get(&url).await.unwrap().text().await.unwrap();
    let jsonval = json::parse(&response).unwrap();
    let mut snowdaypoints: f32 = 0f32;
    let mut temp = 0f64;
    let mut temp2;
    // add all snowfall values to snowdaypoints
    for i in 0..48 {
        let temp_2m = jsonval["hourly"]["snowfall"][i].as_f64().unwrap();
        temp += temp_2m;
    }
    if temp > 0f64 {
        println!("Added 3 points for snowfall initial");
        snowdaypoints += 3f32;
    }
    println!("Added {:?} points for snowfall", temp as i32);
    snowdaypoints += (temp as f32) / 2f32;
    temp = 0f64;

    // add average temp of each hour
    temp2 = 1000f64;
    for i in 0..48 {
        let temp_2m = jsonval["hourly"]["temperature_2m"][i].as_f64().unwrap();
        if temp_2m < temp2 {
            temp2 = temp_2m;
        }
        temp += temp_2m;
    }
    temp /= 48f64;
    if temp < 0f64 {
        println!("Added 2 points for average temp initial");
        snowdaypoints += 2f32;
    }
    if temp2 < -8f64 {
        println!(
            "Added 7 points for lowest temp initial and {:?} points for lowest temp",
            (temp2 as f32).abs()
        );
        snowdaypoints += 7f32 + (temp2 as f32).abs();
    }
    for i in 0..48 {
        let temp_2m = jsonval["hourly"]["visibility"][i].as_f64().unwrap();
        if temp_2m < 1000f64 {
            println!("Added 1 point for low visibility");
            snowdaypoints += 1f32;
            if temp_2m < 100f64 {
                println!("Added 5 points for very low visibility");
                snowdaypoints += 5f32;
                if temp_2m < 10f64 {
                    println!("Added 10 points for extremely low visibility");
                    snowdaypoints += 10f32;
                }
            }
        }
    }
    temp = 0f64;
    temp2 = 0f64;
    for i in 0..48 {
        let temp_2m = jsonval["hourly"]["wind_speed_10m"][i].as_f64().unwrap();
        if temp < temp_2m {
            temp = temp_2m;
        }
        temp2 += temp_2m;
    }
    if temp2 > 20f64 {
        println!(
            "Added {} points for high average wind speed",
            (temp2 as i32) / 200
        );
        snowdaypoints += (temp2 as f32) / 200f32;
    }
    if temp > 20f64 {
        println!("Added 5 points for high wind speed");
        snowdaypoints += 5f32;
    }
    for i in 0..48 {
        if jsonval["hourly"]["precipitation_probability"][i]
            .as_f64()
            .unwrap()
            > 50f64
        {
            println!("Added 1 point for high precipitation probability");
            snowdaypoints += 1f32;
            break;
        }
    }
    temp = 10000f64;
    for i in 0..48 {
        if jsonval["hourly"]["apparent_temperature"][i]
            .as_f64()
            .unwrap()
            < temp
        {
            temp = jsonval["hourly"]["apparent_temperature"][i]
                .as_f64()
                .unwrap();
        }
    }
    if temp < 10f64 {
        println!("Added 2 points for low apparent temperature");
        snowdaypoints += 2f32;
    }
    snowdaypoints -= 15f32; // to  account for normal weather
    if snowdaypoints < 1f32 {
        snowdaypoints = 0.01f32;
    }
    if snowdaypoints > 99f32 {
        snowdaypoints = 99.99f32;
    }
    println!("{:?}% chance of snow day!", snowdaypoints);
    return Ok(snowdaypoints);
}
