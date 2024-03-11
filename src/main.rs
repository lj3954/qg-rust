mod utils;
mod basic_distros;

use reqwest::header::HeaderMap;
use utils::{Distro, collect_distros, DistroError, handle_download, Validation, verify_image};


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
    let (url, iso) = distro.get_url_iso();
    let vm_path = match edition.as_str() {
        "" => format!("{}-{}", os, release),
        _ => format!("{}-{}-{}", os, release, edition),
    };
    std::fs::create_dir(&vm_path).unwrap_or(());
    let path = vm_path.clone() + "/" +  iso.as_str();

    println!("URL: {}, VM_PATH:   {}", url, vm_path);
    let download = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            handle_download(url, path, HeaderMap::new()).await
        })
    });
    let checksum = match distro.has_checksum() {
        true => distro.checksum().unwrap_or_else(|| {
            eprintln!("ERROR: Unable to get checksum. The image will be unable to be verified.");
            "".to_string()
        }),
        _ => "".to_string(),
    };
    let path = match download.join().expect("ERROR: Download thread panicked") {
        Ok(result) => result,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        },
    };

    if checksum.len() > 0 {
        println!("Verifying image with checksum {}", &checksum);
        match verify_image(path, checksum) {
            Ok(true) => println!("Successfully verified image."),
            Ok(false) => eprintln!("ERROR! Image verification failed."),
            Err(e) => eprintln!("WARNING! {}", e),
        }
    }
}

fn get_args() -> (String, String, String) {
    let blank = String::from("");
    let args: Vec<String> = std::env::args().collect();
    let os = args.get(1).unwrap_or(&blank);
    let release = args.get(2).unwrap_or(&blank);
    let edition = args.get(3).unwrap_or(&blank);
   (os.to_lowercase(), release.to_string(), edition.to_string())
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