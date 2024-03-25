// This file contains the logic used for downloading files, 
// as well as for the VM creation.
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, header::HeaderMap};
use crate::utils::{Distro, Config};
use std::fs;
use sha1::Sha1;
use sha2::{Sha256, Sha512, Digest};
use md5::Md5;
use std::error::Error;

pub async fn handle_download(url: String, vm_path: String, headermap: HeaderMap) -> Result<String, std::io::Error> {
    let client = Client::new();
    let path = std::env::current_dir()?.join(vm_path.clone());

    let request = client.get(url).headers(headermap).send().await
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Unable to send request"))?;
    let file_size = request.content_length().unwrap_or(0);

    let progress = ProgressBar::new(file_size);
    progress.set_style(ProgressStyle::with_template("[{elapsed}] {bar:40} {eta_precise} {decimal_bytes}/{decimal_total_bytes}  -   {decimal_bytes_per_sec}")
        .unwrap().progress_chars("##-"));

    let mut stream = request.bytes_stream();
    let mut file = tokio::fs::File::create(&path).await.expect("Unable to create file");

    while let Some(Ok(chunk)) = futures::StreamExt::next(&mut stream).await {
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
        progress.inc(chunk.len() as u64);
    }
    progress.finish();
    Ok(vm_path)
}

pub fn spawn_downloads(url_iso_list: Vec<(String, HeaderMap, String)>, vm_path: &str, distro: &Distro, release: &str, edition: &str, arch: &str) -> Vec<String> {
    println!("Downloading images to {}", vm_path);
    let mut paths = Vec::new();
    for (url, headers, iso) in url_iso_list {
        let path = vm_path.to_string() + iso.as_str();
        let download = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                handle_download(url, path, headers).await
            })
        });
        let checksum = match distro.has_checksum(paths.len()) {
            true => distro.get_checksum(release, edition, arch).unwrap_or("".to_string()),
            _ => "".to_string(),
        };

        let path = match download.join().expect("ERROR: Download thread panicked") {
            Ok(result) => result,
            Err(e) => {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            },
        };

        paths.push(path.clone());

        if checksum.len() > 0 {
            println!("Verifying image with checksum {}", &checksum);
            match verify_image(path, checksum) {
                Ok(true) => println!("Successfully verified image."),
                Ok(false) => {
                    eprintln!("ERROR! Image verification failed.");
                    std::process::exit(1);
                },
                Err(e) => eprintln!("WARNING! {}", e),
            }
        }
    }
    paths
}

pub fn verify_image(filepath: String, checksum: String) -> Result<bool, String> {
    let hash = match checksum.len() {
        32 => {
            let bytes = fs::read(&filepath).map_err(|_| format!("Unable to find MD5sum for file {}", filepath))?;
            hex::encode(Md5::digest(bytes))
        },
        40 => {
            let bytes = fs::read(&filepath).map_err(|_| format!("Unable to find SHA1sum for file {}", filepath))?;
            hex::encode(Sha1::digest(bytes))
        },
        64 => {
            let bytes = fs::read(&filepath).map_err(|_| format!("Unable to find SHA256sum for file {}", filepath))?;
            hex::encode(Sha256::digest(bytes))
        },
        128 => {
            let bytes = fs::read(&filepath).map_err(|_| format!("Unable to find SHA512sum for file {}", filepath))?;
            hex::encode(Sha512::digest(bytes))
        },
        _ => return Err(format!("Can't guess hash algorithm, not checking {} hash.", filepath)),
    };
    Ok(hash == checksum)
}

pub fn create_config(vm_path: &str, paths: Vec<String>, distro: &Distro, release: &str, edition: &str) -> Result<String, Box<dyn Error>> {
    let config_path = vm_path.replace("/", ".conf");
    let path = std::path::Path::new(&config_path);

    let quickemu_path = if let Ok(system_path) = std::env::var("PATH") {
        match system_path.split(':').find(|path| std::path::Path::new(path).join("quickemu").exists()) {
            Some(path) => "#!".to_string() + path + "/quickemu --vm\n".into(),
            _ => "".into()
        }
    } else {
        "".into()
    };

    let default_config = |distro: &Distro| {
        let (os, imagetype) = match distro.name.as_str() {
            "batocera" => ("batocera", "img"),
            "dragonflybsd" => ("dragonflybsd", "iso"),
            "freebsd"|"ghostbsd" => ("freebsd", "iso"),
            "haiku" => ("haiku", "iso"),
            "freedos" => ("freedos", "iso"),
            "kolibrios" => ("kolibrios", "iso"),
            "macos" => ("macos", "img"),
            "netbsd" => ("netbsd", "iso"),
            "openbsd" => ("openbsd", "iso"),
            "openindiana" => ("solaris", "iso"),
            "reactos" => ("reactos", "iso"),
            "truenas" => ("truenas", "iso"),
            "windows" => ("windows", "iso"),
            _ => ("linux", "iso"),
        };


        format!(r#"{}guest_os="{}"
disk_img="{}disk.qcow2"
{}="{}"
arch="{}"
"#, quickemu_path, os, vm_path, imagetype, &paths[0], &distro.arch)
        };



    match distro.config {
        Config::Overwrite(get_config) => {
            let config = get_config(paths, release, edition, &distro.arch)?;
            fs::write(&path, quickemu_path + &config)?;
        },
        Config::Addition(get_addition) => {
            let default = default_config(&distro);
            let addition = get_addition(paths, release, edition, &distro.arch);
            fs::write(&path, default + &addition)?;
        },
        _ => {
            fs::write(&path, default_config(&distro))?;
        },
    }

    Ok(path.to_str().unwrap().to_string())
}

