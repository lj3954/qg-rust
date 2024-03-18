use crate::utils::{Distro, FormatUrl};

pub trait BasicDistros {
    fn add_arch(&mut self, url_format: &str, name: &str, releases: Vec<&str>, editions: Vec<&str>, arch: &str,  checksum: Option<fn(release: &str, edition: &str, arch: &str) -> Option<String>>, pretty_name: &str);

    fn add_edition(&mut self, url_format: &str, name: &str, releases: Vec<&str>, editions: Vec<&str>, checksum: Option<fn(release: &str, edition: &str, arch: &str) -> Option<String>>, pretty_name: &str);
    fn add(&mut self, url_format: &str, name: &str, releases: Vec<&str>, checksum: Option<fn(release: &str, edition: &str, arch: &str) -> Option<String>>, pretty_name: &str);
    fn add_uniqueedition(&mut self, url_format: &str, name: &str, re: Vec<(&str, Vec<&str>)>, checksum: Option<fn(&str, &str, &str) -> Option<String>>, pretty_name: &str);
}

impl BasicDistros for Vec<Distro> {
    fn add_arch(&mut self, url_format: &str, name: &str, releases: Vec<&str>, editions: Vec<&str>, arch: &str, checksum: Option<fn(release: &str, edition: &str, arch: &str) -> Option<String>>, pretty_name: &str) {
        for release in releases {
            for edition in &editions {
                let distro = Distro::Basic {
                    url: url_format.format(release, edition, arch),
                    name: name.to_string(),
                    release: release.to_string(),
                    edition: edition.to_string(),
                    arch: arch.to_string(),
                    checksum,
                    pretty_name: match pretty_name {
                        "" => name.to_string(),
                        _ => pretty_name.to_string(),
                    }
                };
                self.push(distro);
            }
        }
    }

    fn add_edition(&mut self, url_format: &str, name: &str, releases: Vec<&str>, editions: Vec<&str>, checksum: Option<fn(release: &str, edition: &str, arch: &str) -> Option<String>>, pretty_name: &str) {
        self.add_arch(url_format, name, releases, editions, "x86_64", checksum, pretty_name);
    }
    fn add(&mut self, url_format: &str, name: &str, releases: Vec<&str>, checksum: Option<fn(&str, &str, &str) -> Option<String>>, pretty_name: &str) {
        self.add_arch(url_format, name, releases, vec![""], "x86_64", checksum, pretty_name);
    }
    fn add_uniqueedition(&mut self, url_format: &str, name: &str, re: Vec<(&str, Vec<&str>)>, checksum: Option<fn(&str, &str, &str) -> Option<String>>, pretty_name: &str) {
        for (release, editions) in re {
            self.add_edition(url_format, name, vec![release], editions, checksum, pretty_name);
        }
    }

}
