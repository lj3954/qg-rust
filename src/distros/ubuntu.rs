use std::error::Error;
use crate::utils::{FormatUrl, collect_page};
use itertools::Itertools;
use serde::Deserialize;
use rayon::prelude::*;

pub fn get_ubuntu_data(os: &str, release: &str, arch: &str) -> Result<(String, String), Box<dyn Error>> {
    let ubuntu_arch = match arch {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => arch,
    };
    let url = match release {
        "daily-live" => "https://cdimage.ubuntu.com/{EDITION}/{RELEASE}/current/".format(release, os, ubuntu_arch),
        _ if arch != "x86_64" && { os == "ubuntu" || os == "ubuntu-server" } => "https://cdimage.ubuntu.com/releases/{RELEASE}/release/".format(release, os, ubuntu_arch),
        _ if os == "ubuntu" || os == "ubuntu-server" => "https://releases.ubuntu.com/{RELEASE}/".format(release, os, ubuntu_arch), 
        _ => "https://cdimage.ubuntu.com/{EDITION}/releases/{RELEASE}/release/".format(release, os, ubuntu_arch),
    };
    let (imagetype, sku) = match os {
        "ubuntu-server" if arch == "riscv64" => (".img.gz", "live-server"),
        "ubuntu-server" => (".iso", "live-server"),
        "ubuntustudio" => (".iso", "dvd"),
        _ => (".iso", "desktop"),
    };

    let data = match collect_page(url.to_owned() + "SHA256SUMS") {
        Ok(data) => data,
        Err(_) => collect_page(url.to_owned() + "MD5SUMS")?,
    };
    let data = data.lines().find(|line| line.contains(ubuntu_arch) && line.contains(imagetype) && line.contains(sku)).ok_or("Could not find data for architecture.")?;
    let hash = data.split_whitespace().nth(0).ok_or("Could not parse data.")?;
    let iso = url + data.split("*").nth(1).ok_or("Could not parse data.")?;

    Ok((iso, hash.to_owned()))
}

fn get_ubuntu_releases(os: &str, arch: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    let data = collect_page("https://api.launchpad.net/devel/ubuntu/series".to_owned())?;
    let releases: Vec<Entry> = serde_json::from_str::<LaunchpadEntry>(&data)?.entries;
    
    let mut supported = releases.into_iter()
        .filter(|entry| entry.status == "Supported" || entry.status == "Current Stable Release")
        .map(|entry| entry.version)
        .sorted()
        .collect::<Vec<String>>();
    supported.push("daily-live".to_owned());

    if arch != "x86_64" {
        supported = supported.par_iter().filter(|release| {
            get_ubuntu_data(os, release, arch).is_ok()
        }).cloned().collect::<Vec<_>>();
    }
        

    Ok((supported, vec![]))
}

#[derive(Deserialize)]
struct LaunchpadEntry {
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    version: String,
    status: String,
}

pub fn ubuntu_url(release: &str, _: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec![get_ubuntu_data("ubuntu", release, arch)?.0])
}
pub fn ubuntu_checksum(release: &str, _: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    Ok(get_ubuntu_data("ubuntu", release, arch)?.1)
}
pub fn ubuntu_releases(arch: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    get_ubuntu_releases("ubuntu", arch)
}

pub fn kubuntu_url(release: &str, _: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec![get_ubuntu_data("kubuntu", release, arch)?.0])
}
pub fn kubuntu_checksum(release: &str, _: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    Ok(get_ubuntu_data("kubuntu", release, arch)?.1)
}
pub fn kubuntu_releases(arch: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    get_ubuntu_releases("kubuntu", arch)
}

pub fn xubuntu_url(release: &str, _: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec![get_ubuntu_data("xubuntu", release, arch)?.0])
}
pub fn xubuntu_checksum(release: &str, _: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    Ok(get_ubuntu_data("xubuntu", release, arch)?.1)
}
pub fn xubuntu_releases(arch: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    get_ubuntu_releases("xubuntu", arch)
}

pub fn lubuntu_url(release: &str, _: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec![get_ubuntu_data("lubuntu", release, arch)?.0])
}
pub fn lubuntu_checksum(release: &str, _: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    Ok(get_ubuntu_data("lubuntu", release, arch)?.1)
}
pub fn lubuntu_releases(arch: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    get_ubuntu_releases("lubuntu", arch)
}

pub fn ubuntu_budgie_url(release: &str, _: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec![get_ubuntu_data("ubuntu-budgie", release, arch)?.0])
}
pub fn ubuntu_budgie_checksum(release: &str, _: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    Ok(get_ubuntu_data("ubuntu-budgie", release, arch)?.1)
}
pub fn ubuntu_budgie_releases(arch: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    get_ubuntu_releases("ubuntu-budgie", arch)
}

pub fn ubuntu_mate_url(release: &str, _: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec![get_ubuntu_data("ubuntu-mate", release, arch)?.0])
}
pub fn ubuntu_mate_checksum(release: &str, _: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    Ok(get_ubuntu_data("ubuntu-mate", release, arch)?.1)
}
pub fn ubuntu_mate_releases(arch: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    get_ubuntu_releases("ubuntu-mate", arch)
}

pub fn ubuntu_studio_url(release: &str, _: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec![get_ubuntu_data("ubuntustudio", release, arch)?.0])
}
pub fn ubuntu_studio_checksum(release: &str, _: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    Ok(get_ubuntu_data("ubuntustudio", release, arch)?.1)
}
pub fn ubuntu_studio_releases(arch: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    get_ubuntu_releases("ubuntustudio", arch)
}

pub fn ubuntu_cinnamon_url(release: &str, _: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec![get_ubuntu_data("ubuntucinnamon", release, arch)?.0])
}
pub fn ubuntu_cinnamon_checksum(release: &str, _: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    Ok(get_ubuntu_data("ubuntucinnamon", release, arch)?.1)
}
pub fn ubuntu_cinnamon_releases(arch: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    get_ubuntu_releases("ubuntucinnamon", arch)
}

pub fn ubuntu_unity_url(release: &str, _: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec![get_ubuntu_data("ubuntu-unity", release, arch)?.0])
}
pub fn ubuntu_unity_checksum(release: &str, _: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    Ok(get_ubuntu_data("ubuntu-unity", release, arch)?.1)
}
pub fn ubuntu_unity_releases(arch: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    get_ubuntu_releases("ubuntu-unity", arch)
}

pub fn edubuntu_url(release: &str, _: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec![get_ubuntu_data("edubuntu", release, arch)?.0])
}
pub fn edubuntu_checksum(release: &str, _: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    Ok(get_ubuntu_data("edubuntu", release, arch)?.1)
}
pub fn edubuntu_releases(arch: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    get_ubuntu_releases("edubuntu", arch)
}

pub fn ubuntu_kylin_url(release: &str, _: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec![get_ubuntu_data("ubuntukylin", release, arch)?.0])
}
pub fn ubuntu_kylin_checksum(release: &str, _: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    Ok(get_ubuntu_data("ubuntukylin", release, arch)?.1)
}
pub fn ubuntu_kylin_releases(arch: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    get_ubuntu_releases("ubuntukylin", arch)
}

pub fn ubuntu_server_url(release: &str, _: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec![get_ubuntu_data("ubuntu-server", release, arch)?.0])
}
pub fn ubuntu_server_checksum(release: &str, _: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    Ok(get_ubuntu_data("ubuntu-server", release, arch)?.1)
}
pub fn ubuntu_server_releases(arch: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    get_ubuntu_releases("ubuntu-server", arch)
}
