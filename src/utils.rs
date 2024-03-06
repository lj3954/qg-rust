use std::error::Error;
use std::fs::File;
use crate::basic_distros;
use chksum::{md5, sha1, sha2_256, sha2_512};
#[derive(Debug)]
pub enum Distro {
    Basic {
        url: String,
        name: String,
        release: String,
        edition: String,
        arch: String,
        checksum: Option<fn(&str, &str, &str) -> Option<String>>
    },
}

pub fn verify_image(filepath: String, checksum: String) -> Result<bool, String> {
    let file = File::open(filepath.clone()).map_err(|e| format!("ERROR: Unable to open file {}", filepath))?;
    let status = match checksum.len() {
        32 => md5::chksum(file)
            .map_err(|_| format!("ERROR: Unable to get md5sum for file {}", filepath))?
            .to_hex_lowercase() == checksum,
        40 => sha1::chksum(file)
            .map_err(|_| format!("ERROR: Unable to get sha1sum for file {}", filepath))?
            .to_hex_lowercase() == checksum,
        64 => sha2_256::chksum(file)
            .map_err(|_| format!("ERROR: Unable to get sha256sum for file {}", filepath))?
            .to_hex_lowercase() == checksum,
        128 => sha2_512::chksum(file)
            .map_err(|_| format!("ERROR: Unable to get sha512sum for file {}", filepath))?
            .to_hex_lowercase() == checksum,
        _ => return Err("ERROR: Invalid checksum length".to_string()),
    };
    Ok(status)
}

pub trait format_URL {
    fn format(&self, release: &str, edition: &str, arch: &str) -> String;
}

impl format_URL for &str {
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

    // Test cases for checksums
    for distro in &distros {
        match distro {
            Distro::Basic { url, name, release, edition, arch, checksum} => {
                if let Some(getHash) = checksum {
                    let checksum = getHash(release, edition, arch).unwrap();
                    println!("{} {} {} {} hash: {}", name, release, edition, arch, checksum);
                }
            }
            _ => ()
        }
    }

    Err("Error".to_string())
}