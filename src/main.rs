mod models;
mod modules;

extern crate core;

use crate::modules::utils;
use anyhow::{anyhow, Result};
use clap::{arg, App};
use std::process::exit;

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(error) = run().await {
        eprintln!("Error: {error}");
        exit(1);
    }
    Ok(())
}

async fn run() -> Result<()> {
    let app = App::new("bob")
        .subcommand(App::new("use").arg(arg!([VERSION]).required(true)))
        .subcommand(App::new("ls"))
        .get_matches();

    if let Some(subcommand) = app.subcommand_matches("use") {
        if let Err(error) = modules::use_handler::start(subcommand).await {
            return Err(anyhow!(error));
        }
    }

    if let Some(_) = app.subcommand_matches("ls") {
        let downloads_dir = match utils::get_downloads_folder().await {
            Ok(value) => value,
            Err(error) => return Err(anyhow!(error)),
        };

        let paths = std::fs::read_dir(downloads_dir)?;
        let mut versions = String::new();
        let installed_version = match utils::get_current_version().await {
            Some(value) => value,
            None => return Err(anyhow!("Neovim is not installed"))
        };
        let mut first = true;
        for path in paths {
            let path_name = path.unwrap().file_name();
            if !first {
                if path_name.to_str().unwrap().contains(&installed_version) {
                    versions += &format!(", {}(installed)", path_name.to_str().unwrap());
                } else {
                    versions += &format!(", {}", path_name.to_str().unwrap());
                }
            } else {
                if path_name.to_str().unwrap().contains(&installed_version) {
                    versions += &format!("{}(installed)", path_name.to_str().unwrap());
                } else {
                    versions += &format!("{}", path_name.to_str().unwrap());
                }
                first = false;
            }
        }
        println!("{versions}");
    }
    Ok(())
}
