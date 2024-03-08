mod utils;
mod basic_distros;

use reqwest::header::HeaderMap;
use utils::{Distro, collect_distros, ArgsError, handle_download};


fn main() {
    let distros = collect_distros().unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    });
    let (os, release, edition) = match get_args() {
        Ok(args) => args,
        Err(ArgsError::NoOS) => {
            eprintln!("ERROR: No OS specified");
            println!("{}", list_oses(distros));
            std::process::exit(1);
        },
        Err(ArgsError::NoRelease(os)) => {
            if validate_os(&os, &distros) {
                eprintln!("ERROR: No release specified");
                println!("{}", list_releases(&os, distros));
                std::process::exit(1);
            }
            eprintln!("ERROR: Invalid OS");
            println!("{}", list_oses(distros));
            std::process::exit(1);
        },
    };
    let distro = find_matching(&os, &release, &edition, distros).unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    });

    println!("{:?}", distro);

    println!("OS: {}, Release: {}, Edition: {}", os, release, edition);
    let (url, iso) = match distro {
        Distro::Basic { url, name, release, edition, arch, checksum } => {
            (url, format!("{}{}{}.iso", name, release, edition))
        },
        _ => {
            eprintln!("ERROR: Unsupported distro type");
            std::process::exit(1);
        },
    };
    let vm_path = match edition.as_str() {
        "" => format!("{}-{}", os, release),
        _ => format!("{}-{}-{}", os, release, edition),
    };
    std::fs::create_dir(&vm_path).unwrap_or(());
    let vm_path = vm_path + "/" +  iso.as_str();

    println!("URL: {}, VM_PATH:   {}", url, vm_path);
    let download = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            handle_download(url, &vm_path, HeaderMap::new()).await.unwrap();
        });
    });
    download.join().unwrap();
}

fn get_args() -> Result<(String, String, String), ArgsError> {
    let blank = String::from("");
    let args: Vec<String> = std::env::args().collect();
    let os = args.get(1).ok_or(ArgsError::NoOS)?;
    let release = args.get(2).ok_or(ArgsError::NoRelease(os.to_string()))?;
    let edition = args.get(3).unwrap_or(&blank);
   Ok((os.to_string(), release.to_string(), edition.to_string()))
}

fn find_matching(os: &str, chosen_release: &str, chosen_edition: &str, distros: Vec<Distro>) -> Result<Distro, String> {
    let mut matches = (false, false, false);
    for distro in &distros {
        match &distro {
            Distro::Basic { url, name, release, edition, arch, checksum } => {
                matches = (matches.0 || name == os, matches.1 || name == os && release == chosen_release, matches.2 || name == os && matches.1 && edition == chosen_edition);
                if name == os  && release == chosen_release && edition == chosen_edition {
                    return Ok(distro.clone());
                }
            },
            _ => (),
        }
    }
    let err = match matches {
        (true, true, false) => {
            let editions = list_editions(&os, &chosen_release, &distros);
            format!("Edition not found.\n{} {} Editions: {}", os, chosen_release, editions)
        },
        (true, false, _) => format!("Release not found.\n\n{}", list_releases(os, distros)),
        (false, _, _) => {
            let oses = list_oses(distros);
            format!("ERROR: OS not found.\n Supported operating systems: {}", oses)
        },
        _ => "".to_string(),
    };
    Err(err)
}

fn validate_os(os: &str, distros: &Vec<Distro>) -> bool {
    for distro in distros {
        match &distro {
            Distro::Basic { url, name, release, edition, arch, checksum } => {
                if name == os {
                    return true;
                }
            },
            _ => (),
        }
    }
    false
}

fn list_oses(distros: Vec<Distro>) -> String {
    let mut oses = String::new();
    for distro in distros {
        match &distro {
            Distro::Basic { url, name, release, edition, arch, checksum } => {
                if !oses.contains(name) {
                    oses.push_str(name);
                    oses.push_str(" ");
                }
            }
        }
    }
    oses
}
fn list_releases(os: &str, distros: Vec<Distro>) -> String {
    let mut releases = String::new();
    for distro in &distros {
        match &distro {
            Distro::Basic { url, name, release, edition, arch, checksum } => {
                if name == os && !releases.contains(release) {
                    releases.push_str(format!("{}    {}\n", release, list_editions(os, release, &distros)).as_str());
                }
            },
            _ => (),
        }
    }
    releases
}

fn list_editions(os: &str, chosen_release: &str, distros: &Vec<Distro>) -> String {
    let mut editions = String::new();
    for distro in distros {
        match &distro {
            Distro::Basic { url, name, release, edition, arch, checksum } => {
                if name == os && release == chosen_release {
                    editions.push_str(&edition);
                    editions.push_str(" ");
                }
            }
        }
    }
    editions
}













