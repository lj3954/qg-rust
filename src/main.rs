mod utils;
mod distros;

use reqwest::header::HeaderMap;
use utils::{Distro, handle_download, Validation, verify_image};


fn main() {
    let distros = distros::distros();
    let (os, release, edition, download_type, arch) = get_args();
    let distro = distros.validate_parameters(&os, &release, &edition, &arch);

    println!("{:?}", distro);


    println!("OS: {}, Release: {}, Edition: {}", os, release, edition);


    match download_type {
        DownloadType::Normal(vm_path) => {
            let url_iso_list = distro.get_url_iso(&release, &edition, &arch);
            if vm_path.len() > 0 {
                std::fs::create_dir(&vm_path).unwrap_or(());
            }
            spawn_downloads(url_iso_list, vm_path, distro, &release, &edition, &arch)
        },
        DownloadType::Test => {
            let url_iso_list = distro.get_url_iso(&release, &edition, &arch);
            println!("PLACEHOLDER");
            std::process::exit(1);
        },
        DownloadType::Show => {
            let url_iso_list = distro.get_url_iso(&release, &edition, &arch);
            friendly_urls(url_iso_list);
        },
        DownloadType::Homepage => {
            println!("PLACEHOLDER");
            std::process::exit(1);
        },
        _ => {
            eprintln!("ERROR: Invalid download type.");
            std::process::exit(1);
        },
    }

}

fn get_args() -> (String, String, String, DownloadType, String) {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut download_type = DownloadType::None;
    let mut osinfo = Vec::new();
    let mut arch = std::env::consts::ARCH.to_string();

    while let Some(arg) = args.get(0) {
        match arg.as_str() {
            "-h" | "--help" => usage(0),
            "--test-iso-url" | "-t" => download_type = DownloadType::Test,
            "--show-iso-url" | "-s" => download_type = DownloadType::Show,
            "--download-iso" | "-d" => download_type = DownloadType::Normal("".into()),
            "--open-distro-homepage" | "-o" => download_type = DownloadType::Homepage,
            "--arch" | "-a" => {
                if args.len() > 1 {
                    arch = args.remove(1).to_string();
                } else {
                    eprintln!("ERROR: No architecture specified.");
                    usage(1);
                }
            },
            _ => osinfo.push(arg.to_string()),
        }
        args.remove(0);
    };

    if osinfo.len() > 3 {
        eprintln!("ERROR! Too many arguments.");
        usage(1);
    } else if osinfo.len() > 0 {
        if let DownloadType::None = download_type {
            let vm_path = osinfo.iter().map(|s| s.to_owned() + "-").collect::<String>();
            download_type = DownloadType::Normal(format!("{}/", vm_path[0..vm_path.len()-1].to_string()));
        }
    }

    for _ in osinfo.len()..3 {
        osinfo.push("".into());
    }

    println!("{:?}", osinfo);

    (osinfo[0].to_lowercase(), osinfo[1].clone(), osinfo[2].clone(), download_type, arch.into())
}

enum DownloadType {
    None,
    Normal(String),
    Test,
    Show,
    Homepage,
}

fn usage(status: i32) {
    std::process::exit(status);
}

fn spawn_downloads(url_iso_list: Vec<(String, HeaderMap, String)>, vm_path: String, distro: Distro, release: &str, edition: &str, arch: &str) {
    println!("Downloading images to {}", vm_path);
    for (url, headers, iso) in url_iso_list {
        let path = vm_path.clone() + iso.as_str();
        let download = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                handle_download(url, path, headers).await
            })
        });
        let checksum = match distro.has_checksum() {
            true => distro.get_checksum(release, edition, arch).unwrap_or_else(|| {
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
                Ok(false) => {
                    eprintln!("ERROR! Image verification failed.");
                    std::process::exit(1);
                },
                Err(e) => eprintln!("WARNING! {}", e),
            }
        }
    }
}

fn friendly_urls(url_iso_list: Vec<(String, HeaderMap, String)>) {
    println!("{}", url_iso_list.iter().map(|(url, ..)| url.to_string()).collect::<Vec<_>>().join("\n"));
    std::process::exit(1);
}
