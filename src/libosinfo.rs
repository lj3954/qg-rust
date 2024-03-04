use std::process::{Command, Stdio};
use std::fs;
use std::path::Path;

pub struct OS {
    pub name: String,
    pub version: Option<String>,
    pub url: String,
    pub arch: String,
}

pub fn gather_osinfo() -> Result<(Vec<OS>), String> {
    let mut oses:Vec<OS> = Vec::new();


    let temp = Command::new("mktemp").arg("-d").output()
        .map_err(|e| format!("Failed to create a temporary file: {}", e))?.stdout;
    let tempdir = String::from_utf8(temp).map_err(|_| "Failed to parse temporary file name.".to_string())?;
    let path = Path::new(tempdir.trim()).join("osinfo-db.tar.xz");

    let mut dl = downloader::Downloader::builder().build().unwrap();
    let file = downloader::Download::new("https://releases.pagure.org/libosinfo/osinfo-db-20231215.tar.xz");
    println!("Downloading osinfo-db-20231215.tar.xz to {}", tempdir.trim());
    let mut file = file.file_name(&path);
    dl.download(&[file]).map_err(|e| format!("Failed to download the osinfo database: {}", e))?;



    Err("Returned".to_string())
}