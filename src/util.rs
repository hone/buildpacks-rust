use flate2::read::GzDecoder;
use std::{
    fs::File,
    io::{self, Seek, SeekFrom},
    path::Path,
};
use tar::Archive;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("could not download file")]
    Network(#[from] reqwest::Error),
    #[error("could not write file")]
    Io(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum ExtractError {}

pub fn download(url: &str, dst: &mut File) -> Result<u64, DownloadError> {
    let mut resp = reqwest::blocking::get(url)?;
    Ok(resp.copy_to(dst)?)
}

pub fn extract_tar_gz<P: AsRef<Path>>(file: &mut File, dst: P) -> io::Result<()> {
    file.seek(SeekFrom::Start(0))?;
    let tar = GzDecoder::new(file);
    let mut archive = Archive::new(tar);
    archive.unpack(dst)?;
    Ok(())
}
