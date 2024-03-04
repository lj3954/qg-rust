#[derive(Debug)]
pub struct Distro {
    url: String,
    name: String,
    release: String,
    edition: String,
    arch: String,
}

impl Distro {
    fn new(distros: &mut Vec<Distro>, url_format: &str, name: &str, releases: Vec<&str>, editions: Vec<&str>, arch: &str)   {
        for release in releases {
            for edition in &editions {
                let distro = Distro {
                    url: url_format.replace("{RELEASE}", release).replace("{EDITION}", edition),
                    name: name.to_string(),
                    release: release.to_string(),
                    edition: edition.to_string(),
                    arch: arch.to_string(),
                };
                distros.push(distro);
            }
        }
        // distros
    }
    // Create a new Distro with Edition and Arch set to blank/default values
    fn new_noea(distros: &mut Vec<Distro>, url_format: &str, name: &str, releases: Vec<&str>)  {
        Distro::new(distros, url_format, name, releases, vec![""], "x86_64");
    }
}

pub fn collect_distros() -> Result<Vec<Distro>, String> {
    let mut distros: Vec<Distro> = Vec::new();
    Distro::new(&mut distros, "https://zrn.co/{RELEASE}{EDITION}", "Zorin OS", vec!["16", "17"], vec!["core64", "lite64", "education64", "edulite64"], "x86_64");
    Distro::new_noea(&mut distros, "https://files.kde.org/neon/images/{RELEASE}/current/neon-{RELEASE}-current.iso", "KDE Neon", vec!["user", "testing", "unstable", "developer"]);
    println!("{:?}", distros);

    Err("Error".to_string())
}