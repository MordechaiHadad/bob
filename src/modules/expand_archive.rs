use crate::models::DownloadedVersion;
use anyhow::Result;
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

// TODO: Refactor

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
    pb.set_message("Expanding archive");

    std::fs::create_dir(downloaded_file.file_name.clone())?;

    let mut downloaded: u64 = 0;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let temp = &format!("{}/{}", downloaded_file.file_name, file.name());
        let outpath = Path::new(temp);

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

#[cfg(target_family = "unix")] // I don't know if its worth making both expand functions into one function, but the API difference will cause so much if statements
fn expand(downloaded_file: DownloadedVersion) -> Result<()> {
    use flate2::read::GzDecoder;
    use std::os::unix::fs::PermissionsExt;
    use tar::Archive;

    let file = File::open(format!(
        "{}.{}",
        downloaded_file.file_name, downloaded_file.file_format
    ))?;
    let decompress_stream = GzDecoder::new(file);
    let mut archive = Archive::new(decompress_stream);

    let totalsize = 1692; // hard coding this is pretty unwise, but you cant get the length of an archive in tar-rs unlike zip-rs
    let pb = ProgressBar::new(totalsize);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:>7}")
        .progress_chars("█  "));
    pb.set_message("Expanding archive");

    let mut downloaded: u64 = 0;
    for file in archive.entries()? {
        match file {
            Ok(mut file) => {
                let temp = &format!("{}/{}", downloaded_file.file_name, file.path()?.display());
                let outpath = Path::new(temp);

                let file_name = format!("{}", file.path()?.display()); // file.path()?.is_dir() always returns false... weird
                if file_name.ends_with("/") {
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
            Err(error) => println!("{error}"),
        }
    }
    pb.finish_with_message(format!(
        "Finished expanding to {}/{}",
        downloaded_file.path, downloaded_file.file_name
    ));
    let platform = if cfg!(target_os = "macos") {
        "nvim-osx64"
    } else {
        "nvim-linux64"
    };
    let file = &format!("{}/{platform}/bin/nvim", downloaded_file.file_name);
    let mut perms = fs::metadata(file)?.permissions();
    perms.set_mode(0o111);
    fs::set_permissions(file, perms)?;
    Ok(())
}
