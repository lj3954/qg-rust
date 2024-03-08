mod utils;
mod basic_distros;

use reqwest::header::HeaderMap;
use utils::{Distro, collect_distros, DistroError, handle_download, Validation};


fn main() {
    let distros = collect_distros().unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    });
    let (os, release, edition) = get_args();
    let distro = match find_distro(&os, &release, &edition, distros) {
        Ok(distro) => distro,
        Err(DistroError(e, s)) => {
            eprintln!("{}", e);
            println!("{}", s);
            std::process::exit(1);
        },
    };

    println!("{:?}", distro);


    println!("OS: {}, Release: {}, Edition: {}", os, release, edition);
    match &distro {
        Distro::Basic {name, release, edition, arch, pretty_name, .. } => {
            let edition = " ".to_string() + edition;
            if pretty_name.len() > 0 {
                println!("Downloading {} {}{} {}...", pretty_name, release, edition, arch);
            } else {
                println!("Downloading {} {}{} {}...", name, release, edition, arch);
            }
        },
        _ => (),
    }
    let (url, iso) = match distro {
        Distro::Basic { url, name, release, edition, .. } => {
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

fn get_args() -> (String, String, String) {
    let blank = String::from("");
    let args: Vec<String> = std::env::args().collect();
    let os = args.get(1).unwrap_or(&blank);
    let release = args.get(2).unwrap_or(&blank);
    let edition = args.get(3).unwrap_or(&blank);
   (os.to_string(), release.to_string(), edition.to_string())
}

fn find_distro(os: &str, release: &str, edition: &str, distros: Vec<Distro>) -> Result<Distro, DistroError> {
    if os.len() == 0 {
        return Err(DistroError("ERROR! You must specify an operating system.".to_string(), format!(" - Operating systems: {}", distros.list_oses())));
    }
    let pretty_name = match distros.validate_os(&os) {
        (true, pretty_name) => pretty_name,
        (false, _) => return Err(DistroError(format!("ERROR! {} is not a supported OS.", os), format!(" - Operating systems: {}", distros.list_oses()))),
    };
    if release.len() == 0 {
        return Err(DistroError("ERROR! You must specify a release.".to_string(), format!(" - Releases: {}", distros.list_releases(os))));
    } else if !&distros.validate_release(&os, &release) {
        return Err(DistroError(format!("ERROR! {} {} is not a supported release.", pretty_name, release), format!(" - Releases: {}", distros.list_releases(os))));
    }
    return match distros.validate_edition(&os, &release, &edition) {
        Some(distro) => Ok(distro),
        None => match edition.len() {
            0 => Err(DistroError("ERROR! You must specify an edition.".to_string(), format!(" - Editions: {}", distros.list_editions(os, release)))),
            _ => Err(DistroError(format!("ERROR! {} is not a supported {} {} edition", edition, pretty_name, release), format!(" - Editions: {}", distros.list_editions(os, release)))),
        }
    };
}

//     else if !&distros.validate_edition(&os, &release, &edition) {
//         return match edition.len() {
//             0 => Err(DistroError("ERROR! You must specify an edition.".to_string(), format!(" - Editions: {}", distros.list_editions(os, release)))),
//             _ => Err(DistroError(format!("ERROR! {} is not a supported {} {} edition", edition, pretty_name, release), format!(" - Editions: {}", distros.list_editions(os, release)))),
//         }
//     }
// // let (valid_os, pretty_name) = &distros.validate_os(&os);
// // if !valid_os {
// //     return Err(DistroError(format!("ERROR! {} is not a supported OS.", os), format!(" - Operating systems: {}", distros.list_oses())));
// // }

// fn find_matching(os: &str, chosen_release: &str, chosen_edition: &str, distros: Vec<Distro>) -> Result<Distro, String> {
//     let mut matches = (false, false, false);
//     for distro in &distros {
//         match &distro {
//             Distro::Basic { url, name, release, edition, arch, checksum } => {
//                 matches = (matches.0 || name == os, matches.1 || name == os && release == chosen_release, matches.2 || name == os && matches.1 && edition == chosen_edition);
//                 if name == os  && release == chosen_release && edition == chosen_edition {
//                     return Ok(distro.clone());
//                 }
//             },
//             _ => (),
//         }
//     }
//     let err = match matches {
//         (true, true, false) => {
//             let editions = list_editions(&os, &chosen_release, &distros);
//             format!("Edition not found.\n{} {} Editions: {}", os, chosen_release, editions)
//         },
//         (true, false, _) => format!("Release not found.\n\n{}", list_releases(os, distros)),
//         (false, _, _) => {
//             let oses = list_oses(distros);
//             format!("ERROR: OS not found.\n Supported operating systems: {}", oses)
//         },
//         _ => "".to_string(),
//     };
//     Err(err)
// }
//
// fn validate_os(os: &str, distros: &Vec<Distro>) -> bool {
//     for distro in distros {
//         match &distro {
//             Distro::Basic { url:_, name, release:_, edition:_, arch:_, checksum:_ } => {
//                 if name == os {
//                     return true;
//                 }
//             },
//             _ => (),
//         }
//     }
//     false
// }
//
// fn list_oses(distros: Vec<Distro>) -> String {
//     let mut oses = String::new();
//     for distro in distros {
//         match &distro {
//             Distro::Basic { url:_, name, release:_, edition:_, arch:_, checksum:_ } => {
//                 if !oses.contains(name) {
//                     oses.push_str(name);
//                     oses.push_str(" ");
//                 }
//             }
//         }
//     }
//     oses
// }
// fn list_releases(os: &str, distros: Vec<Distro>) -> String {
//     let mut releases = String::new();
//     for distro in &distros {
//         match &distro {
//             Distro::Basic { url:_, name, release, edition:_, arch:_, checksum:_ } => {
//                 if name == os && !releases.contains(release) {
//                     releases.push_str(format!("{}    {}\n", release, list_editions(os, release, &distros)).as_str());
//                 }
//             },
//             _ => (),
//         }
//     }
//     releases
// }
//
// fn list_editions(os: &str, chosen_release: &str, distros: &Vec<Distro>) -> String {
//     let mut editions = String::new();
//     for distro in distros {
//         match &distro {
//             Distro::Basic { url:_, name, release, edition, arch:_, checksum:_ } => {
//                 if name == os && release == chosen_release {
//                     editions.push_str(&edition);
//                     editions.push_str(" ");
//                 }
//             }
//         }
//     }
//     editions
// }



// if os.len() == 0 {
//     eprintln!("ERROR! You must specify an operating system.");
//     println!(" - Operating Systems: {}", distros.list_oses());
// } else if !&distros.validate_os(&os) {
//     eprintln!("ERROR! {} is not a supported OS.", os);
//     println!(" - Operating Systems: {}", distros.list_oses());
// } else if release.len() == 0 {
//     eprintln!("ERROR! You must specify a release.");
//     println!(" - Releases: {}", distros.list_releases(&os));
// } else if !&distros.validate_release(&os, &release) {
//     eprintln!("ERROR! {} {} is not a supported release.");
//     println!(" - Releases: {}", distros.list_releases(&os));
// }
// else if !&distros.validate_release(&os, &release) {
//     eprintln!("ERROR! You must specify a valid release.");
// println!(" - Releases: {}", list_releases(&os, distros));
// }

// let (os, release, edition) = match get_args() {
//     Ok(args) => args,
//     Err(ArgsError::NoOS) => {
//         eprintln!("ERROR: No OS specified");
//         println!("{}", list_oses(distros));
//         std::process::exit(1);
//     },
//     Err(ArgsError::NoRelease(os)) => {
//         if validate_os(&os, &distros) {
//             eprintln!("ERROR: No release specified");
//             println!("{}", list_releases(&os, distros));
//             std::process::exit(1);
//         }
//         eprintln!("ERROR: Invalid OS");
//         println!("{}", list_oses(distros));
//         std::process::exit(1);
//     },
// };
// let distro = find_matching(&os, &release, &edition, distros).unwrap_or_else(|e| {
//     eprintln!("ERROR: {}", e);
//     std::process::exit(1);
// });












