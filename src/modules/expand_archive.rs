use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn start() -> Result<()> {
    expand_unix().await?;
    Ok(())
}

#[cfg(target_family = "windows")]
async fn expand_windows() -> Result<()> {
    use async_zip::read::seek::ZipFileReader;

    let mut file = fs::File::open("v0.6.1.zip").await?;
    let mut zip = ZipFileReader::new(&mut file).await.unwrap();

    for entry in zip.entries() {
        let outpath = entry.name();

        if entry.dir() {
            fs::create_dir(outpath).await?;

        } else {
            let mut file = fs::File::create(outpath).await?;
        }
    }


    Ok(())
}

#[cfg(target_family = "unix")]
async fn expand_unix() -> Result<()> {
    use async_tar::Archive;

    let mut archive = Archive::new("v0.6.1.tar.gz");
    let mut entries = archive.entries()?;

    while let Some(value) = entries.next().await {
        let file = value?;
        println!("{}", file.path().unwrap().display());
    }
    Ok(())
}