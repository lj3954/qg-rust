use std::error::Error;
use std::fs;
use crate::basic_distros;
use sha1::Sha1;
use sha2::{Sha256, Sha512, Digest};
use md5::Md5;
use reqwest::header::HeaderMap;
use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Clone)]
pub enum Distro {
    Basic {
        url: String,
        name: String,
        release: String,
        edition: String,
        arch: String,
        checksum: Option<fn(&str, &str, &str) -> Option<String>>,
        pretty_name: String,
    },
}

impl Distro {
    pub fn url(&self) -> String {
        match self {
            Distro::Basic { url, .. } => url.to_string()
        }
    }
    pub fn has_checksum(&self) -> bool {
        match self {
            Distro::Basic { checksum, .. } => checksum.is_some()
        }
    }
    pub fn checksum(&self) -> Option<String> {
        match self {
            Distro::Basic { release, edition, arch, checksum, .. } => {
                if let Some(get_hash) = checksum {
                    return get_hash(release, edition, arch);
                }
            }
        }
        None
    }
}

pub trait Validation {
    fn validate_os(&self, os: &str) -> (bool, &str);
    fn validate_release(&self, os: &str, release: &str) -> bool;
    fn validate_edition(&self, os: &str, release: &str, edition: &str) -> Option<Distro>;
    fn list_oses(&self) -> String;
    fn list_releases(&self, os: &str) -> String;
    fn list_editions(&self, os: &str, chosen_release: &str) -> String;
}

impl Validation for Vec<Distro> {
    fn validate_os(&self, os: &str) -> (bool, &str) {
        for distro in self {
            match distro {
                Distro::Basic { name, pretty_name, .. } => {
                    if name == os {
                        return (true, pretty_name);
                    }
                }
            }
        }
        (false, "")
    }

    fn validate_release(&self, os: &str, release: &str) -> bool {
        for distro in self {
            match distro {
                Distro::Basic { name, release: distro_release, .. } => {
                    if name == os && distro_release == release {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn validate_edition(&self, os: &str, release: &str, edition: &str) -> Option<Distro> {
        for distro in self {
            match &distro {
                Distro::Basic { name, release: distro_release, edition: distro_edition, .. } => {
                    if name == os && distro_release == release && distro_edition == edition {
                        return Some(distro.clone());
                    }
                }
            }
        }
        None
    }

    fn list_oses(&self) -> String {
        let mut oses = String::new();
        for distro in self {
            match distro {
                Distro::Basic { name, .. } => {
                    if !oses.contains(name) {
                        oses.push_str(name);
                        oses.push_str(" ");
                    }
                }
            }
        }
        oses
    }

    fn list_releases(&self, os: &str) -> String {
        let mut matching_releases: Vec<&str> = Vec::new();
        let mut release_type = "";
        for distro in self {
            match distro {
                Distro::Basic { name, release, edition, .. } if name == os => {
                    match edition.len() {
                        0 => release_type = "basic",
                        _ => release_type = "basic_edition"
                    }
                    if !matching_releases.contains(&&**release) {
                        matching_releases.push(release);
                        }
                }
                _ => ()
            }
        }

        return match release_type {
            "basic" => matching_releases.join(" "),
            "basic_edition" => matching_releases.join(" ") + "\n - Editions: " + &self.list_editions(os, matching_releases[0]),
            _ => String::new()
        }
    }

    fn list_editions(&self, os: &str, chosen_release: &str) -> String {
        let mut editions = String::new();
        for distro in self {
            match distro {
                Distro::Basic { name, release, edition, .. } => {
                    if name == os && release == chosen_release {
                        editions.push_str(&edition);
                        editions.push_str(" ");
                    }
                }
            }
        }
        editions
    }
}

pub struct DistroError (pub String, pub String);

pub fn verify_image(filepath: String, checksum: String) -> Result<bool, String> {
    let hash = match checksum.len() {
        32 => {
            let bytes = fs::read(&filepath).map_err(|_| format!("Unable to find hash for file {}", filepath))?;
            hex::encode(Md5::digest(bytes))
        },
        40 => {
            let bytes = fs::read(&filepath).map_err(|_| format!("Unable to find hash for file {}", filepath))?;
            hex::encode(Sha1::digest(bytes))
        },
        64 => {
            let bytes = fs::read(&filepath).map_err(|_| format!("Unable to find hash for file {}", filepath))?;
            hex::encode(Sha256::digest(bytes))
        },
        128 => {
            let bytes = fs::read(&filepath).map_err(|_| format!("Unable to find hash for file {}", filepath))?;
            hex::encode(Sha512::digest(bytes))
        },
        _ => return Err(format!("Can't guess hash algorithm, not checking {} hash.", filepath)),
    };
    Ok(hash == checksum)
}

pub trait FormatUrl {
    fn format(&self, release: &str, edition: &str, arch: &str) -> String;
}

impl FormatUrl for &str {
    fn format(&self, release: &str, edition: &str, arch: &str) -> String {
        self.replace("{RELEASE}", release).replace("{EDITION}", edition).replace("{ARCH}", arch)
    }
}

pub fn collect_page(url: String) -> Result<String, Box<dyn Error>> {
    let body = reqwest::blocking::get(url)?.text()?;
    Ok(body)
}

pub fn cut_space(s: &str, n: usize) -> String {
    let s = s.split_whitespace();
    for (i, word) in s.enumerate() {
        if i == n-1 {
            return word.to_string();
        }
    }
    "".to_string()
}

pub fn collect_distros() -> Result<Vec<Distro>, String> {
    let mut distros: Vec<Distro> = Vec::new();
    let mut basic_distros = basic_distros::basic_distros();
    distros.append(&mut basic_distros);

    println!("{:?}", distros);

    // // Test cases for checksums
    // for distro in &distros {
    //     match distro {
    //         Distro::Basic { url, name, release, edition, arch, checksum} => {
    //             if let Some(get_hash) = checksum {
    //                 if let Some(checksum) = get_hash(release, edition, arch) {
    //                     println!("{} {} {} {} hash: {}", name, release, edition, arch, checksum);
    //                 }
    //             }
    //         }
    //         _ => ()
    //     }
    // }

    Ok(distros)
}

pub async fn handle_download(url: String, vm_path: String, headermap: HeaderMap) -> Result<String, std::io::Error> {
    let client = Client::new();
    let path = std::env::current_dir()?.join(vm_path.clone());

    let mut request = client.get(url).headers(headermap).send().await
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

