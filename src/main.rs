#![allow(unused)]
mod error;
mod prelude;
mod utils;

use prelude::*;
use std::fs::DirEntry;
use std::thread;
mod response;
mod server;
use clap::{arg, Arg, ArgAction, Command, Parser};
use response::*;
use std::env;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::{format, io::BufRead, io::BufReader, io::Write, net::TcpListener};

// TODO: Use Tokio
fn main() -> Result<()> {
    let matches = Command::new("server")
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
                .about("Start the server")
                .arg(
                    Arg::new("HOST")
                        .short('H')
                        .long("host")
                        .action(ArgAction::Set)
                        .value_name("HOST")
                        .required(false)
                        .default_value("127.0.0.1")
                        .help("Choose a host"),
                )
                .arg(
                    Arg::new("PORT")
                        .short('P')
                        .long("port")
                        .action(ArgAction::Set)
                        .value_name("PORT")
                        .required(false)
                        .default_value("4221")
                        .help("Choose a port"),
                )
                .arg(
                    Arg::new("DIR")
                        .short('D')
                        .long("dir")
                        .action(ArgAction::Set)
                        .value_name("DIR")
                        .required(false)
                        .default_value(".")
                        .help("Choose a static file directory"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("start", set_matches)) => {
            let host = set_matches.get_one::<String>("HOST").unwrap();
            let port = set_matches.get_one::<String>("PORT").unwrap();
            let directory = set_matches.get_one::<String>("DIR").unwrap();
            let directory = Some(PathBuf::from(directory));
            let directory = directory
                .clone()
                .map(fs::canonicalize)
                .and_then(std::io::Result::ok);

            if directory.is_none() {
                return Err(Error::Generic("DIR Doesn't exist".to_string()));
            }

            let mut s = server::Server::new(host, port);
            let listener = s.start();

            for stream in listener.incoming() {
                let directory = Arc::new(directory.clone());

                thread::spawn(move || {
                    handle_stream(stream, directory);
                });
            }
        }
        _ => {}
    }

    fn handle_stream(
        stream: std::io::Result<std::net::TcpStream>,
        directory: Arc<Option<PathBuf>>,
    ) {
        match stream {
            Ok(stream) => {
                let buf = {
                    let stream = stream.try_clone().expect("failed to clone tcp stream");
                    BufReader::new(stream)
                };

                stream
                    .set_write_timeout(None)
                    .expect("set_write_timeout call failed");

                stream
                    .set_read_timeout(None)
                    .expect("set_read_timeout call failed");

                let mut path = "/".to_string();
                let mut user_agent = String::new();

                'request_parse: for (i, _line) in buf.lines().enumerate() {
                    match _line {
                        Ok(line) => {
                            // perhaps there is no proper EOL with request
                            if line.is_empty() {
                                println!("Request Empty Line");
                                break 'request_parse;
                            }

                            if i == 0 {
                                let mut parts = line
                                    .split_whitespace()
                                    .map(String::from)
                                    .collect::<Vec<String>>();

                                // TODO
                                // method = parts.remove(0);
                                path = parts.remove(1);
                            } else {
                                if line.starts_with("User-Agent:") {
                                    user_agent = line
                                        .strip_prefix("User-Agent: ")
                                        .expect("invalid user agent")
                                        .to_string();
                                }
                            }
                        }
                        _ => {
                            println!("Request EOL");
                            break 'request_parse;
                        }
                    }
                }

                match path.as_str() {
                    "/" => {
                        let response = Response::new("/", 200, "OK");
                        let response = response.content_type("text/plain");
                        let mut response = response.content_length(0);
                        response.send(stream, "");
                    }
                    // TODO:: better routing
                    path if path.starts_with("/files") => {
                        let file_name = path.strip_prefix("/files/").expect("invalid url");

                        let dir = directory.as_ref().clone().unwrap();

                        if let Ok(file) = fs::File::open(dir.join(file_name)) {
                            let mut file_reader = BufReader::new(file);

                            let mut contents = String::new();
                            if let Ok(bytes_read) = file_reader.read_to_string(&mut contents) {
                                let response = Response::new("/files", 200, "OK");
                                let response = response.content_type("application/octet-stream");
                                let mut response = response.content_length(bytes_read);
                                response.send(stream, contents.as_str());
                            } else {
                                let response =
                                    Response::new("/files", 500, "Internal Server Error");
                                let response = response.content_type("application/octet-stream");
                                let mut response = response.content_length(0);
                                response.send(stream, "");
                            }
                        } else {
                            let response = Response::new("/files", 404, "NOT FOUND");
                            let response = response.content_type("application/octet-stream");
                            let mut response = response.content_length(0);
                            response.send(stream, "");
                        }
                    }
                    path if path.starts_with("/user-agent") => {
                        let response = Response::new("/user-agent", 200, "OK");
                        let response = response.content_type("text/plain");
                        let mut response = response.content_length(user_agent.len());
                        response.send(stream, &user_agent)
                    }
                    path if path.starts_with("/echo") => {
                        let body = path.strip_prefix("/echo/").expect("invalid url");

                        let response = Response::new("/echo", 200, "OK");
                        let response = response.content_type("text/plain");
                        let mut response = response.content_length(body.len());
                        response.send(stream, body);
                    }
                    _ => {
                        let response = Response::new(path, 404, "NOT FOUND");
                        let response = response.content_type("text/plain");
                        let mut response = response.content_length(0);
                        response.send(stream, "");
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
