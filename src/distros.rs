mod add_distro;
mod windows;
mod macos;

use crate::utils::{Distro, cut_space, collect_page, FormatUrl, Checksum, URL, ReleaseEdition, Config};
use add_distro::{BasicDistros, AdvancedDistros};
use std::error::Error;
use std::io::ErrorKind;

pub fn distros() -> Vec<Distro> {
    let mut distros = Vec::new();
 
    distros.add_basic("https://neon.kde.org/", "kdeneon", "KDE Neon", vec!["user", "testing", "unstable", "developer"], vec![], "https:files.kde.org/neon/images/{RELEASE}/current/neon-{RELEASE}-current.iso", Checksum::Normal(kdeneon_hash), "x86_64", Config::None);


    distros.add_advanced_unique("https://www.microsoft.com/en-us/windows/", "windows", "Windows", vec![("8", vec!["Arabic", "Brazilian Portuguese", "Bulgarian", "Chinese (Simplified)", "Chinese (Traditional)", "Chinese (Traditional Hong Kong)", "Croatian", "Czech", "Danish", "Dutch", "English (United States)", "English International", "Estonian", "Finnish", "French", "German", "Greek", "Hebrew", "Hungarian", "Italian", "Japanese", "Latvian", "Lithuanian", "Norwegian", "Polish", "Portuguese", "Romanian", "Russian", "Serbian Latin", "Slovak", "Slovenian", "Spanish", "Swedish", "Thai", "Turkish", "Ukrainian"]), 
            ("10", vec!["Arabic", "Brazilian Portuguese", "Bulgarian", "Chinese (Simplified)", "Chinese (Traditional)", "Czech", "Danish", "Dutch", "English (United States)", "English International", "Estonian", "Finnish", "French", "French Canadian", "German", "Greek", "Hebrew", "Hungarian", "Italian", "Japanese", "Korean", "Latvian", "Lithuanian", "Norwegian", "Polish", "Portuguese", "Romanian", "Russian", "Serbian Latin", "Slovak", "Slovenian", "Spanish", "Spanish (Mexico)", "Swedish", "Thai", "Turkish", "Ukrainian"]),
            ("11", vec!["Arabic", "Brazilian Portuguese", "Bulgarian", "Chinese (Simplified)", "Chinese (Traditional)", "Czech", "Danish", "Dutch", "English (United States)", "English International", "Estonian", "Finnish", "French", "French Canadian", "German", "Greek", "Hebrew", "Hungarian", "Italian", "Japanese", "Korean", "Latvian", "Lithuanian", "Norwegian", "Polish", "Portuguese", "Romanian", "Russian", "Serbian Latin", "Slovak", "Slovenian", "Spanish", "Spanish (Mexico)", "Swedish", "Thai", "Turkish", "Ukrainian"])], windows::get_windows_url, Checksum::None, "x86_64", Config::Addition(windows::windows_config));


    distros.add("https://www.apple.com/macos/", "macos", "macOS", ReleaseEdition::Basic(vec!["high-sierra".into(), "mojave".into(), "catalina".into(), "big-sur".into(), "monterey".into(), "ventura".into(), "sonoma".into()], vec![]), URL::PlusHeaders(macos::get_urls), Checksum::Manual(macos::verify_chunklist), "x86_64", Config::Addition(macos::macos_config));

    distros
}

fn kdeneon_hash(release: &str, edition: &str, arch: &str) -> Option<String> {
    match collect_page("https:files.kde.org/neon/images/{RELEASE}/current/neon-{RELEASE}-current.sha256sum".format(release, edition, arch)) {
        Ok(body) if body.len() > 0 => {
            let checksum = cut_space(&body, 1);
            Some(checksum)
        },
        _ => None,
    }
}
