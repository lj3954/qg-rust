mod add_distro;
use crate::utils::{Distro, cut_space, collect_page, FormatUrl, Checksum, URL, ReleaseEdition};
use add_distro::{BasicDistros, AdvancedDistros};

pub fn distros() -> Vec<Distro> {
    let mut distros = Vec::new();
 
    distros.add_basic("kdeneon", "KDE Neon", vec!["user", "testing", "unstable", "developer"], vec![], "https://files.kde.org/neon/images/{RELEASE}/current/neon-{RELEASE}-current.iso", Checksum::Normal(kdeneon_hash), "amd64");
    




    distros
}

fn kdeneon_hash(release: &str, edition: &str, arch: &str) -> Option<String> {
    match collect_page("https://files.kde.org/neon/images/{RELEASE}/current/neon-{RELEASE}-current.sha256sum".format(release, edition, arch)) {
        Ok(body) if body.len() > 0 => {
            let checksum = cut_space(&body, 1);
            Some(checksum)
        },
        _ => None,
    }
}
