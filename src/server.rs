use std::{format, io::BufRead, io::BufReader, io::Write, net::TcpListener};

pub struct Server {
    pub host: String,
    pub port: String,
}

impl Server {
    pub fn new(host: impl Into<String>, port: impl Into<String>) -> Self {
        Server {
            host: host.into(),
            port: port.into(),
        }
    }

    pub fn start(&mut self) -> TcpListener {
        TcpListener::bind(format!("{}:{}", self.host, self.port)).expect("Failed to start server")
    }
}
