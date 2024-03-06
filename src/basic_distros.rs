use crate::utils::{Distro, cut_space, collect_page, format_URL};

pub trait DistroVec {
    fn add_arch(&mut self, url_format: &str, name: &str, releases: Vec<&str>, editions: Vec<&str>, arch: &str,  checksum: Option<fn(release: &str, edition: &str, arch: &str) -> Option<String>>);

    fn add_edition(&mut self, url_format: &str, name: &str, releases: Vec<&str>, editions: Vec<&str>, checksum: Option<fn(release: &str, edition: &str, arch: &str) -> Option<String>>);
    fn add(&mut self, url_format: &str, name: &str, releases: Vec<&str>, checksum: Option<fn(release: &str, edition: &str, arch: &str) -> Option<String>>);
}

impl DistroVec for Vec<Distro> {
    fn add_arch(&mut self, url_format: &str, name: &str, releases: Vec<&str>, editions: Vec<&str>, arch: &str, checksum: Option<fn(release: &str, edition: &str, arch: &str) -> Option<String>>) {
        for release in releases {
            for edition in &editions {
                let distro = Distro::Basic {
                    url: url_format.format(release, edition, arch),
                    name: name.to_string(),
                    release: release.to_string(),
                    edition: edition.to_string(),
                    arch: arch.to_string(),
                    checksum,
                };
                self.push(distro);
            }
        }
    }

    fn add_edition(&mut self, url_format: &str, name: &str, releases: Vec<&str>, editions: Vec<&str>, checksum: Option<fn(release: &str, edition: &str, arch: &str) -> Option<String>>) {
        self.add_arch(url_format, name, releases, editions, "x86_64", checksum);
    }
    fn add(&mut self, url_format: &str, name: &str, releases: Vec<&str>, checksum: Option<fn(&str, &str, &str) -> Option<String>>) {
        self.add_arch(url_format, name, releases, vec![""], "x86_64", checksum);
    }
}

pub fn basic_distros() -> Vec<Distro> {
    let mut distros: Vec<Distro> = Vec::new();
    // URL Format : Pretty Name : Releases : (Editions) : Checksum function : (Arch)
    distros.add_edition("https://zrn.co/{RELEASE}{EDITION}", "Zorin OS", vec!["16", "17"], vec!["core64", "lite64", "education64", "edulite64"], None);
    distros.add("https://files.kde.org/neon/images/{RELEASE}/current/neon-{RELEASE}-current.iso", "KDE Neon", vec!["user", "testing", "unstable", "developer"], Some(kdeneon_hash));


    distros
}

fn kdeneon_hash(release: &str, edition: &str, arch: &str) -> Option<String> {
    match collect_page("https://files.kde.org/neon/images/{RELEASE}/current/neon-{RELEASE}-current.sha256sum".format(release, edition, arch)) {
        Ok(body) if body.len() > 0 => {
            let checksum = cut_space(&body, 1);
            Some(checksum)
        },
        _ => {
            println!("ERROR: Unable to get KDE Neon {} checksum", release);
            None
        },
    }
}