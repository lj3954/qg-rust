use crate::utils::collect_page;
use std::error::Error;
use serde::Deserialize;
use itertools::Itertools;

#[derive(Debug, Deserialize)]
struct FedoraRelease {
    version: String,
    arch: String,
    link: String,
    subvariant: String,
    sha256: Option<String>,
}

pub fn fedora_releases(arch: &str) -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>> {
    let json = collect_page("https://getfedora.org/releases.json".into())?;
    let json: Vec<FedoraRelease> = serde_json::from_str(&json)?;

    let release_edition = json.into_iter().filter_map(|entry| {
        if entry.arch == arch {
            Some((entry.version, entry.subvariant))
        } else {
            None
        }
    })
    .dedup()
    .group_by(|entry| entry.0.clone())
    .into_iter()
    .map(|(release, editions)| (release, editions.map(|value| value.1).collect::<Vec<String>>()))
    .collect::<Vec<(String, Vec<String>)>>();
    Ok(release_edition)
}

pub fn get_fedora_urls(release: &str, edition: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let json = collect_page("https://getfedora.org/releases.json".into())?;
    let json: Vec<FedoraRelease> = serde_json::from_str(&json)?;

    let urls = json.into_iter().filter_map(|entry| {
        if entry.version == release && entry.subvariant == edition && entry.arch == arch {
            Some(entry.link)
        } else {
            None
        }
    })
    .find_or_first(|url| url.ends_with(".iso"));

    match urls {
        Some(url) => Ok(vec![url]),
        None => Err("Could not find URL".into())
    }
}

pub fn fedora_checksum(release: &str, edition: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    let json = collect_page("https://getfedora.org/releases.json".into())?;
    let json: Vec<FedoraRelease> = serde_json::from_str(&json)?;

    let checksum = json.into_iter().filter_map(|entry| {
        if entry.version == release && entry.subvariant == edition && entry.arch == arch {
            Some((entry.link, entry.sha256))
        } else {
            None
        }
    })
    .find(|checksum| checksum.0.ends_with(".iso"));

    match checksum {
        Some(checksum) => match checksum {
            (_, Some(checksum)) => Ok(checksum),
            _ => Err(format!("Checksum is not available for Fedora {} {} {}", release, edition, arch).into())
        },
        None => Err(format!("Checksum is not available for Fedora {} {} {}", release, edition, arch).into())
    }
}
