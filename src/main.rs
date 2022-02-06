use std::fs::File;
use std::num::ParseFloatError;
use serde::{Serialize, Deserialize};
use std::process::Command;
use clap::{App, Arg, arg, ArgMatches};
use regex::Regex;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new("bob").subcommand(App::new("use")
        .arg(
            arg!([VERSION])
                .required(true)))
        .get_matches();

    if let Some(subcommand) = app.subcommand_matches("use") {
        if let Some(value) = subcommand.value_of("VERSION") {
            let version = String::from(value);
            let version = parse_version(version).await?;
            download_version(&version).await?;

        }
    }
    Ok(())
}

async fn download_version(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(format!("https://github.com/neovim/neovim/releases/download/{}/nvim-win64.zip", version))
        .header("user-agent", "bob")
        .send()
        .await;

    match response {
        Ok(response) => {
            let response_bytes = response.bytes().await?;
            if String::from_utf8_lossy(&response_bytes) != "Not Found" {
                let mut file = tokio::fs::File::create("bro.zip").await?;
                file.write_all(&response_bytes).await;
                println!("Successfully downloaded version {}", version);
            } else {println!("Please provide an existing version");}
        },
        Err(error) => println!("{}", error)
    }
    Ok(())
}

async fn parse_version<'a>(mut version: String) -> Result<String, &'a str> {
    match &*version {
        "nightly" | "stable" => Ok(version),
        _ => {
            let regex = Regex::new(r"[0-9]*\.[0-9]*\.[0-9]*").unwrap();
            if regex.is_match(&*version) {
                if !version.contains("v") { version = format!("v{}", version);}
                return Ok(version)
            }
            Err("Please provide a proper version string")
        }
    }
}