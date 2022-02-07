use anyhow::{anyhow, Result};
use futures_util::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::cmp::min;
use tokio::io::AsyncWriteExt;

pub async fn start(version: &str) -> Result<()> {
    if let Err(error) = download_version(version).await {
        return Err(anyhow!("{error}"));
    }
    Ok(())
}

async fn download_version(version: &str) -> Result<()> {
    let response = send_request(version).await;

    match response {
        Ok(response) => {
            let total_size = response.content_length().unwrap();
            let response_status = response.status();
            let mut response_bytes = response.bytes_stream();
            if response_status == 200 {
                let pb = ProgressBar::new(total_size);
                pb.set_style(ProgressStyle::default_bar()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                    .progress_chars("█  "));
                pb.set_message(format!("Downloading version: {version}"));
                let mut file =
                    tokio::fs::File::create(format!("{version}.{}", get_file_type().await)).await?;

                let mut downloaded: u64 = 0;

                while let Some(item) = response_bytes.next().await {
                    let chunk = item.or(anyhow::private::Err(anyhow::Error::msg("hello")))?;
                    file.write(&chunk).await;
                    let new = min(downloaded + (chunk.len() as u64), total_size);
                    downloaded = new;
                    pb.set_position(new);
                }

                pb.finish_with_message(format!("Downloaded version {version}"));
                Ok(())
            } else {
                Err(anyhow!("Please provide an existing neovim version"))
            }
        }
        Err(error) => Err(anyhow!(error)),
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
        "https://github.com/neovim/neovim/releases/download/{version}/nvim-{os}.{}",
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
