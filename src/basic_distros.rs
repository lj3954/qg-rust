use crate::utils::{Distro, cut_space, collect_page, FormatUrl};
pub mod add_distro;
use add_distro::BasicDistros;

// Available methods:
// add_arch(url format, name, releases vector, editions vector, arch, checksum function, pretty name)
// add_edition(url format, name, releases vector, editions vector, checksum function, pretty name)
// add(url format, name, releases, checksum function, pretty name)
// add_uniqueedition(url format, name, release/edition vector, checksum function, pretty name)
// 
// These functions can be used with the same values multiple times. If you need to add multiple
// architectures, you can use add_arch after already using add_uniqueedition, for example

pub fn basic_distros() -> Vec<Distro> {
    let mut distros: Vec<Distro> = Vec::new();
    distros.add_uniqueedition("https://zrn.co/{RELEASE}{EDITION}", "zorin", vec![("16", vec!["core64", "lite64", "education64", "edulite64"]), ("17", vec!["core64"])], None, "Zorin OS");
    distros.add("https://files.kde.org/neon/images/{RELEASE}/current/neon-{RELEASE}-current.iso", "kdeneon", vec!["user", "testing", "unstable", "developer"], Some(kdeneon_hash), "KDE Neon");


    distros
}

// Below here, add functions used for finding checksums. These must take in a release, edition, and
// architecture, and return Some(checksum) or None. 

fn kdeneon_hash(release: &str, edition: &str, arch: &str) -> Option<String> {
    match collect_page("https://files.kde.org/neon/images/{RELEASE}/current/neon-{RELEASE}-current.sha256sum".format(release, edition, arch)) {
        Ok(body) if body.len() > 0 => {
            let checksum = cut_space(&body, 1);
            Some(checksum)
        },
        _ => None,
    }
}
