use crate::utils::{Distro, Checksum, URL, ReleaseEdition};
use std::error::Error;

pub trait BasicDistros {
    fn add_basic(&mut self, name: &str, pretty_name: &str, releases: Vec<&str>, editions: Vec<&str>, url_format: &str, checksum: Checksum, arch: &str);
    fn add_unique(&mut self, name: &str, pretty_name: &str, release_edition: Vec<(&str, Vec<&str>)>, url_format: &str, checksum: Checksum, arch: &str);
    fn add_basic_online(&mut self, name: &str, pretty_name: &str, release_editions: fn() -> Result<(Vec<String>, Vec<String>), Box<dyn Error>>, url_format: &str, checksum: Checksum, arch: &str);
    fn add_unique_online(&mut self, name: &str, pretty_name: &str, release_editions: fn() -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>>, url_format: &str, checksum: Checksum, arch: &str);
}

pub trait AdvancedDistros {
    fn add_advanced(&mut self, name: &str, pretty_name: &str, releases: Vec<&str>, editions: Vec<&str>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str);
    fn add_advanced_unique(&mut self, name: &str, pretty_name: &str, release_editions: Vec<(&str, Vec<&str>)>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str);
    fn add_advanced_online(&mut self, name: &str, pretty_name: &str, release_editions: fn() -> Result<(Vec<String>, Vec<String>), Box<dyn Error>>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str);
    fn add_advanced_unique_online(&mut self, name: &str, pretty_name: &str, release_editions: fn() -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str);
    fn add(&mut self, name: &str, pretty_name: &str, release_edition: ReleaseEdition, url: URL, checksum: Checksum, arch: &str);
}

trait FixVec<T> {
    fn fix(self) -> Vec<T>;
}

impl FixVec<String> for Vec<&str> {
    fn fix(self) -> Vec<String> {
        self.into_iter().map(String::from).collect()
    }
}

impl FixVec<(String, Vec<String>)> for Vec<(&str, Vec<&str>)> {
    fn fix(self) -> Vec<(String, Vec<String>)> {
        self.into_iter().map(|(release, editions)| (release.into(), editions.fix())).collect()
    }
}

impl BasicDistros for Vec<Distro> {
    fn add_basic(&mut self, name: &str, pretty_name: &str, releases: Vec<&str>, editions: Vec<&str>, url_format: &str, checksum: Checksum, arch: &str) {
        self.add(name, pretty_name, ReleaseEdition::Basic(releases.fix(), editions.fix()), URL::Format(url_format.into()), checksum, arch);
    }
    fn add_unique(&mut self, name: &str, pretty_name: &str, release_edition: Vec<(&str, Vec<&str>)>, url_format: &str, checksum: Checksum, arch: &str) {
        self.add(name, pretty_name, ReleaseEdition::Unique(release_edition.fix()), URL::Format(url_format.into()), checksum, arch);
    }
    fn add_basic_online(&mut self, name: &str, pretty_name: &str, release_editions: fn() -> Result<(Vec<String>, Vec<String>), Box<dyn Error>>, url_format: &str, checksum: Checksum, arch: &str) {
        self.add(name, pretty_name, ReleaseEdition::OnlineBasic(release_editions), URL::Format(url_format.into()), checksum, arch);
    }
    fn add_unique_online(&mut self, name: &str, pretty_name: &str, release_editions: fn() -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>>, url_format: &str, checksum: Checksum, arch: &str) {
        self.add(name, pretty_name, ReleaseEdition::OnlineUnique(release_editions), URL::Format(url_format.into()), checksum, arch);
    }
}

impl AdvancedDistros for Vec<Distro> {
    fn add_advanced(&mut self, name: &str, pretty_name: &str, releases: Vec<&str>, editions: Vec<&str>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str) {
        self.add(name, pretty_name, ReleaseEdition::Basic(releases.fix(), editions.fix()), URL::Function(url), checksum, arch);
    }
    fn add_advanced_unique(&mut self, name: &str, pretty_name: &str, release_editions: Vec<(&str, Vec<&str>)>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str) {
        self.add(name, pretty_name, ReleaseEdition::Unique(release_editions.fix()), URL::Function(url), checksum, arch);
    }
    fn add_advanced_online(&mut self, name: &str, pretty_name: &str, release_editions: fn() -> Result<(Vec<String>, Vec<String>), Box<dyn Error>>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str) {
        self.add(name, pretty_name, ReleaseEdition::OnlineBasic(release_editions), URL::Function(url), checksum, arch);
    }
    fn add_advanced_unique_online(&mut self, name: &str, pretty_name: &str, release_editions: fn() -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str) {
        self.add(name, pretty_name, ReleaseEdition::OnlineUnique(release_editions), URL::Function(url), checksum, arch);
    }
    fn add(&mut self, name: &str, pretty_name: &str, release_edition: ReleaseEdition, url: URL, checksum: Checksum, arch: &str) {
        self.push(Distro {
            name: name.into(),
            pretty_name: pretty_name.into(),
            release_edition,
            url,
            checksum_function: checksum,
            arch: arch.into(),
        });
    }
}
