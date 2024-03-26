use crate::utils::{Distro, Checksum, URL, ReleaseEdition, Config};
use std::error::Error;

pub trait BasicDistros {
    fn add_basic(&mut self, homepage: &str, name: &str, pretty_name: &str, releases: Vec<&str>, editions: Vec<&str>, url_format: &str, checksum: Checksum, arch: &str, config: Config);
    fn add_unique(&mut self, homepage: &str ,name: &str, pretty_name: &str, release_edition: Vec<(&str, Vec<&str>)>, url_format: &str, checksum: Checksum, arch: &str, config: Config);
    fn add_basic_online(&mut self, homepage: &str, name: &str, pretty_name: &str, release_editions: fn(&str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>>, url_format: &str, checksum: Checksum, arch: &str, config: Config);
    fn add_unique_online(&mut self, homepage: &str, name: &str, pretty_name: &str, release_editions: fn(&str) -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>>, url_format: &str, checksum: Checksum, arch: &str, config: Config);
}

pub trait AdvancedDistros {
    fn add_advanced(&mut self, homepage: &str, name: &str, pretty_name: &str, releases: Vec<&str>, editions: Vec<&str>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str, config: Config);
    fn add_advanced_unique(&mut self, homepage: &str, name: &str, pretty_name: &str, release_editions: Vec<(&str, Vec<&str>)>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str, config: Config);
    fn add_advanced_online(&mut self, homepage: &str, name: &str, pretty_name: &str, release_editions: fn(&str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str, config: Config);
    fn add_advanced_unique_online(&mut self, homepage: &str, name: &str, pretty_name: &str, release_editions: fn(&str) -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str, config: Config);
    fn add(&mut self, homepage: &str, name: &str, pretty_name: &str, release_edition: ReleaseEdition, url: URL, checksum: Checksum, arch: &str, config: Config);
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
    fn add_basic(&mut self, homepage: &str, name: &str, pretty_name: &str, releases: Vec<&str>, editions: Vec<&str>, url_format: &str, checksum: Checksum, arch: &str, config: Config) {
        self.add(homepage, name, pretty_name, ReleaseEdition::Basic(releases.fix(), editions.fix()), URL::Format(url_format.into()), checksum, arch, config);
    }
    fn add_unique(&mut self, homepage: &str, name: &str, pretty_name: &str, release_edition: Vec<(&str, Vec<&str>)>, url_format: &str, checksum: Checksum, arch: &str, config: Config) {
        self.add(homepage, name, pretty_name, ReleaseEdition::Unique(release_edition.fix()), URL::Format(url_format.into()), checksum, arch, config);
    }
    fn add_basic_online(&mut self, homepage: &str, name: &str, pretty_name: &str, release_editions: fn(&str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>>, url_format: &str, checksum: Checksum, arch: &str, config: Config) {
        self.add(homepage, name, pretty_name, ReleaseEdition::OnlineBasic(release_editions), URL::Format(url_format.into()), checksum, arch, config);
    }
    fn add_unique_online(&mut self, homepage: &str, name: &str, pretty_name: &str, release_editions: fn(&str) -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>>, url_format: &str, checksum: Checksum, arch: &str, config: Config) {
        self.add(homepage, name, pretty_name, ReleaseEdition::OnlineUnique(release_editions), URL::Format(url_format.into()), checksum, arch, config);
    }
}

impl AdvancedDistros for Vec<Distro> {
    fn add_advanced(&mut self, homepage: &str, name: &str, pretty_name: &str, releases: Vec<&str>, editions: Vec<&str>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str, config: Config) {
        self.add(homepage, name, pretty_name, ReleaseEdition::Basic(releases.fix(), editions.fix()), URL::Function(url), checksum, arch, config);
    }
    fn add_advanced_unique(&mut self, homepage: &str, name: &str, pretty_name: &str, release_editions: Vec<(&str, Vec<&str>)>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str, config: Config) {
        self.add(homepage, name, pretty_name, ReleaseEdition::Unique(release_editions.fix()), URL::Function(url), checksum, arch, config);
    }
    fn add_advanced_online(&mut self, homepage: &str, name: &str, pretty_name: &str, release_editions: fn(&str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str, config: Config) {
        self.add(homepage, name, pretty_name, ReleaseEdition::OnlineBasic(release_editions), URL::Function(url), checksum, arch, config);
    }
    fn add_advanced_unique_online(&mut self, homepage: &str, name: &str, pretty_name: &str, release_editions: fn(&str) -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>>, url: fn(&str, &str, &str) -> Result<Vec<String>, Box<dyn Error>>, checksum: Checksum, arch: &str, config: Config) {
        self.add(homepage, name, pretty_name, ReleaseEdition::OnlineUnique(release_editions), URL::Function(url), checksum, arch, config);
    }
    fn add(&mut self, homepage: &str, name: &str, pretty_name: &str, release_edition: ReleaseEdition, url: URL, checksum: Checksum, arch: &str, config: Config) {
        self.push(Distro {
            name: name.into(),
            pretty_name: pretty_name.into(),
            release_edition,
            url,
            checksum_function: checksum,
            arch: arch.into(),
            homepage: homepage.into(),
            config,
        });
    }
}
