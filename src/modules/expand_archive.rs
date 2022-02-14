use crate::models::DownloadedVersion;
use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::cmp::min;
use std::fs::File;
use std::path::Path;
use std::{fs, io};

pub async fn start(file: DownloadedVersion) -> Result<()> {
    let temp_file = file.clone();
    tokio::task::spawn_blocking(move || {
        expand(temp_file).unwrap();
    })
    .await?;
    tokio::fs::remove_file(format!(
        "{}/{}.{}",
        file.path, file.file_name, file.file_format
    ))
    .await?;
    Ok(())
}

#[cfg(target_family = "windows")]
fn expand(downloaded_file: DownloadedVersion) -> Result<()> {
    use zip::ZipArchive;

    let file = File::open(format!(
        "{}.{}",
        downloaded_file.file_name, downloaded_file.file_format
    ))?;

    let mut archive = ZipArchive::new(file)?;
    let totalsize: u64 = archive.len() as u64;

    let pb = ProgressBar::new(totalsize);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:>7}")
        .progress_chars("█  "));
    pb.set_message("Unzipping file");

    std::fs::create_dir(downloaded_file.file_name.clone())?;

    let mut downloaded: u64 = 0;
    for i in 0..archive.len() {
        //TODO: find way to optimize
        let mut file = archive.by_index(i)?;
        let temp = format!("{}/{}", downloaded_file.file_name, file.name());
        let outpath = Path::new(temp.as_str());

        if file.is_dir() {
            fs::create_dir_all(outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = fs::File::create(outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
        let new = min(downloaded + 1, totalsize);
        downloaded = new;
        pb.set_position(new);
    }
    pb.finish_with_message(format!(
        "Finished unzipping to {}/{}",
        downloaded_file.path, downloaded_file.file_name
    ));

    Ok(())
}

#[cfg(target_family = "unix")]
fn expand(file: &File) -> Result<()> {
    use tar::Archive;

    let mut archive = tar::Archive::new(file);
    for file in archive.entries()? {
        println!("{}", file?.path()?.file_name().unwrap().to_str().unwrap());
    }

    Ok(())
}
