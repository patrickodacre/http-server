use std::{format, io::Write};

pub struct Response<T, L> {
    route: String,
    data: String,
    content_type: T,
    content_length: L,
}

#[derive(Debug, Clone)]
pub struct NoContentType;
#[derive(Debug, Clone)]
pub struct ContentType(String);
#[derive(Debug, Clone)]
pub struct NoContentLength;
#[derive(Debug, Clone)]
pub struct ContentLength(usize);

impl Response<ContentType, ContentLength> {
    pub fn send(&mut self, mut stream: std::net::TcpStream, body: &str) {
        self.data
            .push_str(&format!("Content-Type: {}\r\n", &self.content_type.0));
        self.data
            .push_str(&format!("Content-Length: {}\r\n", &self.content_length.0));

        self.data.push_str("\r\n");

        self.data.push_str(body);

        let res = self.data.to_owned();
        match stream.write(res.as_bytes()) {
            Ok(bytes_written) => {
                println!("200 /{} : bytes written :: {:?}", self.route, bytes_written);
            }
            _ => {
                println!("Failed to send response {}", self.route);
            }
        }
    }
}

impl<T: Clone> Response<T, NoContentLength> {
    pub fn content_length(&self, len: usize) -> Response<T, ContentLength> {
        Response {
            route: self.route.to_owned(),
            data: self.data.to_owned(),
            content_type: self.content_type.clone(),
            content_length: ContentLength(len),
        }
    }
}

impl<L: Clone> Response<NoContentType, L> {
    pub fn content_type(&self, _type: impl Into<String>) -> Response<ContentType, L> {
        Response {
            route: self.route.to_owned(),
            data: self.data.to_owned(),
            content_type: ContentType(_type.into()),
            content_length: self.content_length.clone(),
        }
    }
}

impl Response<NoContentType, NoContentLength> {
    pub fn new(route: impl Into<String>, status_code: i32, text: &str) -> Self {
        let mut response = String::new();

        response.push_str(&format!("HTTP/1.1 {} {}\r\n", status_code, text));
        response.push_str("Server: RustServer\r\n");
        response.push_str("Connection: Keep-Alive\r\n");
        response.push_str("Keep-Alive: timeout=5, max=1000\r\n");

        Response {
            route: route.into(),
            data: response,
            content_type: NoContentType,
            content_length: NoContentLength,
        }
    }
}
