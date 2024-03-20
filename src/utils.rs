use std::error::Error;
use std::fs;
use sha1::Sha1;
use sha2::{Sha256, Sha512, Digest};
use md5::Md5;
use reqwest::header::HeaderMap;
use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};


#[derive(Debug, Clone)]
pub struct Distro {
    pub pretty_name: String,
    pub name: String,
    pub url: URL,
    pub release_edition: ReleaseEdition,
    pub arch: String,
    pub checksum_function: Checksum,
}

#[derive(Debug, Clone)]
pub enum Checksum {
    None,
    Normal(fn(&str, &str, &str) -> Option<String>),
    Manual(fn(String, &str, &str, &str) -> bool),
}

#[derive(Debug, Clone)]
pub enum URL {
    Format(String),
    Function(fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>),
    PlusHeaders(fn(&str, &str, &str) -> Result<Vec<(String, HeaderMap)>, Box<dyn Error>>),
}

#[derive(Debug, Clone)]
pub enum ReleaseEdition {
    Basic(Vec<String>, Vec<String>),
    Unique(Vec<(String, Vec<String>)>),
    OnlineBasic(fn() -> Result<(Vec<String>, Vec<String>), Box<dyn Error>>),
    OnlineUnique(fn() -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>>),
}

impl Distro {
    pub fn get_url_iso(&self, release: &str, edition: &str, arch: &str) -> Vec<(String, HeaderMap, String)> {
        let image_types = vec![".iso", ".img", ".dmg", ".chunklist", ".xz", ".raw", ".zip", ".tar", ".gz"];

        let Distro { url, name, .. } = self;
        let iso_format = |url: &str| {
            match url.rsplit('/').next() {
                Some(iso) if image_types.iter().any(|&extension| iso.ends_with(extension)) => iso.to_string(),
                _ if edition.len() > 0 => format!("{}-{}-{}.iso", name, release, edition),
                _ => format!("{}-{}.iso", name, release),
            }
        };

        match url {
            URL::Format(URLString) => vec![(URLString.as_str().format(release, edition, arch), HeaderMap::new(), iso_format(URLString))],
            URL::Function(GetURL) => {
                match GetURL(release, edition, arch) {
                    Ok(urls) => urls.iter()
                        .map(|url| (url.to_string(), HeaderMap::new(), iso_format(url)))
                        .collect(),
                    Err(e) => {
                        eprintln!("Unable to get URLs: {}", e);
                        std::process::exit(1);
                    },
                }
            },
            URL::PlusHeaders(GetInfo) => {
                match GetInfo(release, edition, arch) {
                    Ok(urls) => urls.iter()
                        .map(|(url, header)| (url.to_string(), header.clone(), iso_format(url)))
                        .collect(),
                    Err(e) => {
                        eprintln!("Unable to get URLs: {}", e);
                        std::process::exit(1);
                    },
                }
            },
        }
    }
    pub fn has_checksum(&self) -> bool {
        match self.checksum_function {
            Checksum::Normal(_) => true,
            _ => false,
        }
    }

    pub fn get_checksum(&self, release: &str, edition: &str, arch: &str) -> Option<String> {
        match self.checksum_function {
            Checksum::Normal(get_hash) => get_hash(release, edition, arch),
            _ => None,
        }
    }

    pub fn verify_after(&self, path: String, release: &str, edition: &str, arch: &str) -> Option<bool> {
        match self.checksum_function {
            Checksum::Manual(verify) => Some(verify(path, release, edition, arch)),
            _ => None,
        }
    }
}

pub trait Validation {
    fn validate_parameters(&self, os: &str, release: &str, edition: &str) -> Distro;
    fn list_oses(&self) -> String;
    fn list_releases(&self, releases: Vec<(String, Vec<String>)>) -> String;
}

impl Validation for Vec<Distro> {
    fn validate_parameters(&self, os: &str, release: &str, edition: &str) -> Distro {
        if os.len() == 0 {
            eprintln!("ERROR! You must specify an operating system.");
            println!(" - Operating systems: {}", self.list_oses());
            std::process::exit(1);
        }

        let distros = self.iter().filter(|distro| distro.name == os)
            .cloned()
            .collect::<Vec<Distro>>();
        
        if distros.len() == 0 {
            eprintln!("ERROR! {} is not a supported OS.", os);
            println!(" - Operating systems: {}", self.list_oses());
            std::process::exit(1);
        }
        let pretty_name = distros[0].pretty_name.clone();

        let mut data: Vec<(String, Vec<String>)> = Vec::new();

        for distro in distros {
            match &distro.release_edition {
                ReleaseEdition::Basic(releases, editions) => {
                    if releases.contains(&release.to_string()) && editions.len() == 0 || editions.contains(&edition.to_string()) {
                        return distro.clone();
                    }
                    data.append(&mut releases.iter().map(|release| (release.to_string(), editions.clone())).collect());
                },
                ReleaseEdition::Unique(releases) => {
                    if releases.iter().any(|(rel, editions)| rel == release && editions.len() == 0 || editions.contains(&edition.to_string())) {
                        return distro.clone();
                    }
                    data.append(&mut releases.clone());
                },
                ReleaseEdition::OnlineBasic(get_releases) => match get_releases() {
                    Ok((releases, editions)) => {
                        if releases.contains(&release.to_string()) && editions.len() == 0 || editions.contains(&edition.to_string()) {
                            return distro.clone();
                        }
                        data.append(&mut releases.iter().map(|release| (release.to_string(), editions.clone())).collect());
                    },
                    Err(e) => {
                        eprintln!("Unable to get releases for {}: {}", distro.name, e);
                        std::process::exit(1);
                    },
                },
                ReleaseEdition::OnlineUnique(get_info) => match get_info() {
                    Ok(releases) => {
                        if releases.iter().any(|(rel, editions)| rel == release && editions.len() == 0 || editions.contains(&edition.to_string())) {
                            return distro.clone();
                        }
                        data.append(&mut releases.clone());
                    },
                    Err(e) => {
                        eprintln!("Unable to get releases for {}: {}", distro.name, e);
                        std::process::exit(1);
                    },
                },
            }
        }

        if release.len() == 0 {
            eprintln!("ERROR! You must specify a release.");
            println!("{}", self.list_releases(data));
            std::process::exit(1);
        }

        for (release, editions) in &data {
            if release == release {
                if !editions.contains(&edition.to_string()) {
                    eprintln!("ERROR! {} is not a supported {} {} edition", edition, pretty_name, release);
                    println!(" - Editions: {}", editions.join(" "));
                    std::process::exit(1);
                } else {
                    panic!("ERROR! Somehow an OS was not returned despite being found in the list. This should never happen.");
                }
            }
        }
        eprintln!("ERROR! {} is not a supported {} release.", release, pretty_name);
        println!("{}", self.list_releases(data));
        std::process::exit(1);
    }


    fn list_oses(&self) -> String {
        self.iter().map(|distro| distro.name.to_string()).collect::<String>()
    }

    fn list_releases(&self, releases: Vec<(String, Vec<String>)>) -> String {
        if releases.iter().all(|(_, editions)| editions.len() == 0) {
            return format!(" - Releases: {}",  releases.iter().map(|(release, _)| release.to_string()).collect::<Vec<String>>().join(" "));
        } else if releases.iter().all(|(_, editions)| editions == &releases[0].1) {
            return format!(" - Releases: {}\n - Editions: {}", releases.iter().map(|(release, _)| release.to_string()).collect::<Vec<String>>().join(" "), releases[0].1.join(" "));
        } else {
            return releases.iter().map(|(release, editions)| {
                format!("{}     -     {}", release, editions.join(" "))
            }).collect::<Vec<String>>().join("\n");
        }
    }
}

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

