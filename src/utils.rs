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
    // 'Basic' distros have a simple URL format. No requests need to be made to find the URL
    Basic {
        url: String,
        name: String,
        release: String,
        edition: String,
        arch: String,
        checksum: Option<fn(&str, &str, &str) -> Option<String>>,
        pretty_name: String,
    },
    // 'Advanced' distros do not have a simple URL format. The URL is found through a function call
    // These can also have multiple files to download, so the function returns a String vector.
    Advanced {
        urls: fn(&str, &str, &str) -> Vec<String>,
        name: String,
        release: String,
        edition: String,
        arch: String,
        checksum: Option<fn(&str, &str, &str) -> Option<String>>,
        pretty_name: String,
    },
    // 'Custom' distros add a HeaderMap, as well as handling their own image verification
    Custom {
        urls: fn(&str, &str, &str) -> Vec<(String, HeaderMap)>,
        name: String,
        release: String,
        edition: String,
        arch: String,
        checksum: Option<fn(String, &str, &str, &str) -> bool>,
        pretty_name: String,
    }
}

impl Distro {
    pub fn get_url_iso(&self) -> Vec<(String, HeaderMap, String)> {
        let image_types = vec![".iso", ".img", ".dmg", ".chunklist", ".xz", ".raw", ".zip", ".tar", ".gz"];
        let mut list: Vec<(String, HeaderMap, String)> = Vec::new();
        let (name, release, edition, urls) = match self {
            Distro::Basic { url, name, release, edition, .. } => (name, release, edition, vec![(url.to_string(), HeaderMap::new())]),
            Distro::Advanced { urls, name, release, edition, arch, .. } => {
                let urls = urls(&release, &edition, &arch)
                    .iter()
                    .map(|url| (url.to_string(), HeaderMap::new()))
                    .collect();
                (name, release, edition, urls)
            },
            Distro::Custom { urls, name, release, edition, arch, .. } => {
                let urls = urls(&release, &edition, &arch);
                (name, release, edition, urls)
            },
        };
        for (url, header) in urls {
            let iso = match url.rsplit('/').next() {
                Some(iso) if image_types.iter().any(|&extension| iso.ends_with(extension)) => iso.to_string(),
                _ if edition.len() > 0 => format!("{}-{}-{}.iso", name, release, edition),
                _ => format!("{}-{}.iso", name, release),
            };
            list.push((url.to_string(), header, iso));
        }

        list
    }
    pub fn has_checksum(&self) -> bool {
        match self {
            Distro::Basic { checksum, .. } => checksum.is_some(),
            Distro::Advanced { checksum, .. } => checksum.is_some(),
            Distro::Custom { checksum, .. } => checksum.is_some(),
        }
    }
    pub fn checksum(&self) -> Option<String> {
        match self {
            Distro::Basic { release, edition, arch, checksum, .. } => {
                if let Some(get_hash) = checksum {
                    return get_hash(release, edition, arch);
                }
            },
            Distro::Advanced { release, edition, arch, checksum, .. } => {
                if let Some(get_hash) = checksum {
                    return get_hash(release, edition, arch);
                }
            },
            _ => ()
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
                },
                Distro::Advanced { name, pretty_name, .. } => {
                    if name == os {
                        return (true, pretty_name);
                    }
                },
                Distro::Custom { name, pretty_name, .. } => {
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
                Distro::Basic { name, release: distro_release, .. } if name == os && distro_release == release  => return true,
                Distro::Advanced { name, release: distro_release, .. } if name == os && distro_release == release  => return true,
                Distro::Custom { name, release: distro_release, .. } if name == os && distro_release == release  => return true,
                _ => ()
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
                },
                Distro::Advanced { name, release: distro_release, edition: distro_edition, .. } => {
                    if name == os && distro_release == release && distro_edition == edition {
                        return Some(distro.clone());
                    }
                },
                Distro::Custom { name, release: distro_release, edition: distro_edition, .. } => {
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
                },
                Distro::Advanced { name, .. } => {
                    if !oses.contains(name) {
                        oses.push_str(name);
                        oses.push_str(" ");
                    }
                },
                Distro::Custom { name, .. } => {
                    if !oses.contains(name) {
                        oses.push_str(name);
                        oses.push_str(" ");
                    }
                },
            }
        }
        oses
    }

    fn list_releases(&self, os: &str) -> String {
        let mut matching_releases: Vec<String> = Vec::new();
        let mut release_type = "";
        for distro in self {
            let (name, release, edition) = match distro {
                Distro::Basic { name, release, edition, .. } => (name, release, edition),
                Distro::Advanced { name, release, edition, .. } => (name, release, edition),
                Distro::Custom { name, release, edition, .. } => (name, release, edition),
            };
            if name == os {
                if release_type.len() == 0 {
                    match edition.len() {
                        0 => release_type = "basic",
                        _ => release_type = "edition",
                    }
                }
                if !matching_releases.contains(&release) {
                    matching_releases.push(release.to_string());
                }
            }
        }

        match release_type {
            "basic" => matching_releases.join(" "),
            "edition" => {
                let mut editions: Vec<String> = Vec::new();
                for release in &matching_releases {
                    editions.push(self.list_editions(&os, release));
                }
                // Determine whether all editions are the same
                if editions.iter().all(|edition| edition == &editions[0]) {
                    format!("{}\n - Editions: {}", matching_releases.join(" "), editions[0])
                } else {
                    matching_releases.iter().enumerate().map(|(index, release)| {
                        format!("{}     -     {}", release, editions.get(index).unwrap())
                    }).collect::<Vec<String>>().join("\n")
                }
            },
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
                },
                Distro::Advanced { name, release, edition, .. } => {
                    if name == os && release == chosen_release {
                        editions.push_str(&edition);
                        editions.push_str(" ");
                    }
                },
                Distro::Custom { name, release, edition, .. } => {
                    if name == os && release == chosen_release {
                        editions.push_str(&edition);
                        editions.push_str(" ");
                    }
                },
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

