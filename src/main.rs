extern crate core;

use clap::{arg, App, Arg, ArgMatches};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::num::ParseFloatError;
use std::process::{exit, Command};
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new("bob")
        .subcommand(App::new("use").arg(arg!([VERSION]).required(true)))
        .get_matches();

    if let Some(subcommand) = app.subcommand_matches("use") {
        if let Some(value) = subcommand.value_of("VERSION") {
            let version = String::from(value);
            let version = match parse_version(version).await {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("Error: {}", error.as_ref());
                    exit(1);
                }
            };
            if let Err(error) = download_version(&version).await {
                eprintln!("Error: {}", error.as_ref());
                exit(1);
            }
        }
    }
    Ok(())
}
async fn parse_version(mut version: String) -> Result<String, Box<dyn std::error::Error>> {
    match version.as_str() {
        "nightly" | "stable" => Ok(version),
        _ => {
            let regex = Regex::new(r"[0-9]*\.[0-9]*\.[0-9]*").unwrap();
            if regex.is_match(version.as_str()) {
                if !version.contains("v") {
                    version = format!("v{}", version);
                }
                return Ok(version);
            }
            Err(Box::new(Error::new(
                ErrorKind::Other,
                "Please provide a proper version string",
            )))
        }
    }
}

async fn download_version(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = send_request(version).await;

    match response {
        Ok(response) => {
            let response_bytes = response.bytes().await?;
            if String::from_utf8_lossy(&response_bytes) != "Not Found" {
                let mut file =
                    tokio::fs::File::create(format!("{}.{}", version, get_file_type().await))
                        .await?;
                file.write_all(&response_bytes).await;
                println!("Successfully downloaded version {}", version);
                Ok(())
            } else {
                Err(Box::new(Error::new(
                    ErrorKind::Other,
                    "Please provide an existing neovim version",
                )))
            }
        }
        Err(error) => Err(Box::new(Error::new(ErrorKind::Other, error))),
    }
}

async fn send_request(version: &str) -> Result<reqwest::Response, reqwest::Error> {
    let os = if cfg!(target_os = "linux") {
        "linux64"
    } else if cfg!(target_os = "windows") {
        "win64"
    } else {
        "macos"
    };
    let request_url = format!(
        "https://github.com/neovim/neovim/releases/download/{}/nvim-{}.{}",
        version,
        os,
        get_file_type().await
    );

    let client = reqwest::Client::new();
    client
        .get(request_url)
        .header("user-agent", "bob")
        .send()
        .await
}

async fn get_file_type() -> String {
    if cfg!(target_family = "windows") {
        String::from("zip")
    } else {
        String::from("tar.gz")
    }
}
