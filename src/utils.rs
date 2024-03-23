use std::error::Error;
use reqwest::header::HeaderMap;


#[derive(Debug, Clone)]
pub struct Distro {
    pub pretty_name: String,
    pub name: String,
    pub url: URL,
    pub release_edition: ReleaseEdition,
    pub arch: String,
    pub checksum_function: Checksum,
    pub homepage: String,
    pub config: Config,
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

#[derive(Debug, Clone)]
pub enum Config {
    None,
    Addition(fn(Vec<String>, &str, &str, &str) -> String),
    Overwrite(fn(Vec<String>, &str, &str, &str) -> Result<String, Box<dyn Error>>),
}

impl Distro {
    pub fn get_url_iso(&self, release: &str, edition: &str, arch: &str) -> Vec<(String, HeaderMap, String)> {
        let image_types = vec![".iso", ".img", ".dmg", ".chunklist", ".xz", ".raw", ".zip", ".tar", ".gz", ".msi"];

        let Distro { url, name, .. } = self;
        let iso_format = |url: &str| {
            match url.rsplit('/').next() {
                Some(iso) if image_types.iter().any(|&extension| iso.ends_with(extension)) => iso.to_string(),
                _ if edition.len() > 0 => format!("{}-{}-{}.iso", name, release, edition),
                _ => format!("{}-{}.iso", name, release),
            }
        };

        match url {
            URL::Format(url_string) => {
                let url_string = url_string.as_str().format(release, edition, arch);
                let iso = iso_format(&url_string);
                vec![(url_string, HeaderMap::new(), iso)]
            },
            URL::Function(get_url) => {
                match get_url(release, edition, arch) {
                    Ok(urls) => urls.iter()
                        .map(|url| (url.to_string(), HeaderMap::new(), iso_format(url)))
                        .collect(),
                    Err(e) => {
                        eprintln!("Unable to get URLs: {}", e);
                        std::process::exit(1);
                    },
                }
            },
            URL::PlusHeaders(get_info) => {
                match get_info(release, edition, arch) {
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
    pub fn has_checksum(&self, index: usize) -> bool {
        match self.checksum_function {
            Checksum::Normal(_) if index == 0 => true,
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
    fn validate_parameters(&self, os: &str, release: &str, edition: &str, arch: &str) -> Distro;
    fn list_oses(&self) -> String;
    fn list_releases(&self, releases: Vec<(String, Vec<String>)>) -> String;
}

impl Validation for Vec<Distro> {
    fn validate_parameters(&self, os: &str, release: &str, edition: &str, arch: &str) -> Distro {
        if os.len() == 0 {
            eprintln!("ERROR! You must specify an operating system.");
            println!(" - Operating systems: {}", self.list_oses());
            std::process::exit(1);
        }

        let distros:Vec<Distro> = match self.iter().all(|distro| distro.name == os && distro.arch == arch) {
                true => self.iter().filter(|distro| distro.name == os && distro.arch == arch).cloned().collect(),
                false => self.iter().filter(|distro| distro.name == os).cloned().collect(),
        };
        
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
                    if releases.iter().any(|(rel, editions)| rel == release && { editions.len() == 0 || editions.contains(&edition.to_string()) }) {
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

        for (rel, editions) in &data {
            if rel == release {
                if !editions.contains(&edition.to_string()) {
                    if edition.is_empty() {
                        eprintln!("ERROR! You must specify an edition.");
                    } else {
                        eprintln!("ERROR! {} is not a supported {} {} edition", edition, pretty_name, release);
                    }
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
        self.iter().map(|distro| distro.name.to_string()).collect::<Vec<String>>().join(" ")
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


