mod add_distro;
mod windows;
mod macos;

use crate::utils::{Distro, cut_space, collect_page, FormatUrl, Checksum, URL, ReleaseEdition, Config};
use add_distro::{BasicDistros, AdvancedDistros};
use std::error::Error;
use std::io::ErrorKind;

// List of functions used to add distros

// add_basic(homepage, name, pretty_name, releases, editions, url_format, checksum, arch, config)
// add_unique(homepage, name, pretty_name, release_editions, url_format, checksum, arch, config)
// add_basic_online(homepage, name, pretty_name, release_editions, url_format, checksum, arch, config)
// add_unique_online(homepage, name, pretty_name, release_editions, url_format, checksum, arch, config)
//
// add_advanced(homepage, name, pretty_name, releases, editions, url, checksum, arch, config)
// add_advanced_unique(homepage, name, pretty_name, release_editions, url, checksum, arch, config)
// add_advanced_online(homepage, name, pretty_name, release_editions, url, checksum, arch, config)
// add_advanced_unique_online(homepage, name, pretty_name, release_editions, url, checksum, arch, config)

// Information:
// Homepage: self-explanatory. Just the URL to the website.
// Name: Also very self explanatory. Remember, a name shouldn't include special characters/spaces
// Pretty Name: The friendly name of your OS. This can include any characters.
//
// Release formats:
    // "releases, editions": 2 vectors, one for releases and one for editions. If there are no
    // editions, pass an empty vector.
    //
    // "release_editions" for normal 'unique' distros: A vector of tuples, where the first element is
    // the release and the second a list of editions for that release. 
    // For example, you could pass vec![("16", vec!["KDE", "GNOME"]), ("17", vec!["XFCE", "LXQt"])] to
    // include KDE and GNOME editions of release 16, and XFCE and LXQt editions of release 17.
    //
    // "release_editions" for normal online distros: A function which returns a vector of releases and
    // a vector of editions in a tuple. This is used if you can fetch the available releases from the
    // internet. Otherwise, it's similar to the first format.
    //
    // "release_editions" for unique online distros: A function which returns a vector of tuples, where
    // the first element is the release and the second a list of editions for that release. This is
    // used when you have different editions for each release, and you can fetch those from the
    // internet. 
//
// URL types:
    // "url_format": A string which contains {RELEASE}, {EDITION}, and {ARCH} fields as needed,
    // which are replaced with the release, edition, and architecture respectively.
    //
    // "url": A function which takes the release, edition, and architecture as arguments and returns
    // a vector of URLs (surrounded in Ok()), or an error (surrounded in Err())
// Checksum types:
    // "Checksum::None": No checksum is used.
    // "Checksum::Normal(function)": A function takes in the release, edition, and architecture and
    // returns the checksum of the first file downloaded (usually the ISO), or an error.
//
// arch: The architecture of the OS. Use standard names like "x86_64" or "aarch64".
//
// Config types:
    // "Config::None": Use the default configuration
    // Config::Addition(function): A function which takes in the ISO paths, release, edition, and
    // architecture, and returns the lines that need to be added to the configuration file.
    // Config::Overwrite(function): A function which takes in the ISO paths, release, edition, and
    // architecture, and returns the entire config file. 

pub fn distros() -> Vec<Distro> {
    let mut distros = Vec::new();
 
    distros.add_basic("https://neon.kde.org/", "kdeneon", "KDE Neon", vec!["user", "testing", "unstable", "developer"], vec![], "https:files.kde.org/neon/images/{RELEASE}/current/neon-{RELEASE}-current.iso", Checksum::Normal(kdeneon_hash), "x86_64", Config::None);


    distros.add_advanced_unique("https://www.microsoft.com/en-us/windows/", "windows", "Windows", vec![("8", vec!["Arabic", "Brazilian Portuguese", "Bulgarian", "Chinese (Simplified)", "Chinese (Traditional)", "Chinese (Traditional Hong Kong)", "Croatian", "Czech", "Danish", "Dutch", "English (United States)", "English International", "Estonian", "Finnish", "French", "German", "Greek", "Hebrew", "Hungarian", "Italian", "Japanese", "Latvian", "Lithuanian", "Norwegian", "Polish", "Portuguese", "Romanian", "Russian", "Serbian Latin", "Slovak", "Slovenian", "Spanish", "Swedish", "Thai", "Turkish", "Ukrainian"]), 
            ("10", vec!["Arabic", "Brazilian Portuguese", "Bulgarian", "Chinese (Simplified)", "Chinese (Traditional)", "Czech", "Danish", "Dutch", "English (United States)", "English International", "Estonian", "Finnish", "French", "French Canadian", "German", "Greek", "Hebrew", "Hungarian", "Italian", "Japanese", "Korean", "Latvian", "Lithuanian", "Norwegian", "Polish", "Portuguese", "Romanian", "Russian", "Serbian Latin", "Slovak", "Slovenian", "Spanish", "Spanish (Mexico)", "Swedish", "Thai", "Turkish", "Ukrainian"]),
            ("11", vec!["Arabic", "Brazilian Portuguese", "Bulgarian", "Chinese (Simplified)", "Chinese (Traditional)", "Czech", "Danish", "Dutch", "English (United States)", "English International", "Estonian", "Finnish", "French", "French Canadian", "German", "Greek", "Hebrew", "Hungarian", "Italian", "Japanese", "Korean", "Latvian", "Lithuanian", "Norwegian", "Polish", "Portuguese", "Romanian", "Russian", "Serbian Latin", "Slovak", "Slovenian", "Spanish", "Spanish (Mexico)", "Swedish", "Thai", "Turkish", "Ukrainian"])], windows::get_windows_url, Checksum::None, "x86_64", Config::Addition(windows::windows_config));


    distros.add("https://www.apple.com/macos/", "macos", "macOS", ReleaseEdition::Basic(vec!["high-sierra".into(), "mojave".into(), "catalina".into(), "big-sur".into(), "monterey".into(), "ventura".into(), "sonoma".into()], vec![]), URL::PlusHeaders(macos::get_urls), Checksum::Manual(macos::verify_chunklist), "x86_64", Config::Addition(macos::macos_config));

    distros
}

// Available functions:
// collect_page: Takes in a URL and returns the body of the page as a string, or an error.
// cut_space: Cuts to the nth word in a string, starting at 0.
// .format: Formats a string slice with the release, edition, and architecture

fn kdeneon_hash(release: &str, edition: &str, arch: &str) -> Result<String, Box<dyn Error>> {
    let body = collect_page("https:files.kde.org/neon/images/{RELEASE}/current/neon-{RELEASE}-current.sha256sum".format(release, edition, arch))?;
    let checksum = cut_space(&body, 1);
    Ok(checksum)
}
