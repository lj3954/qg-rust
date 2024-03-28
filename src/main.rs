mod utils;
mod distros;
mod quickget;

use reqwest::header::HeaderMap;
use utils::{Distro, Validation, List};
use quickget::{spawn_downloads, create_config, test_urls};


fn main() {
    let distros = distros::distros();
    let (os, release, edition, download_type, arch) = get_args();

    if let DownloadType::List(json) = download_type {
        distros.list(json);
    }

    let distro = distros.validate_parameters(&os, &release, &edition, &arch);
    let arch = &distro.arch;

    //println!("{:?}", distro);


    //println!("OS: {}, Release: {}, Edition: {}", os, release, edition);


    match download_type {
        DownloadType::Normal(vm_path) => {
            let url_iso_list = distro.get_url_iso(&release, &edition, &arch);
            if vm_path.len() > 0 {
                let vm_path = format!("{}{}/", vm_path, distro.arch);
                std::fs::create_dir(&vm_path).unwrap_or(());
                let paths = spawn_downloads(url_iso_list, &vm_path, &distro, &release, &edition, &arch);
                match distro.verify_after(&paths, &release, &edition, &arch) {
                    Some(true) => println!("Successfully verified {} image.", distro.pretty_name),
                    Some(false) => {
                        eprintln!("ERROR: Failed to verify {} image.", distro.pretty_name);
                        std::process::exit(1);
                    },
                    None => (),
                };
                match create_config(&vm_path, paths, &distro, &release, &edition) {
                    Ok(config) => println!("\nTo start your {} virtual machine, run\n    quickemu --vm {}\n",
                                           distro.pretty_name, config),
                    Err(e) => eprintln!("ERROR: {}", e),
                }
            } else {
                spawn_downloads(url_iso_list, &vm_path, &distro, &release, &edition, &arch);
            }
        },
        DownloadType::Test => {
            let url_iso_list = distro.get_url_iso(&release, &edition, &arch);
            test_urls(url_iso_list);
            std::process::exit(0);
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
            "list" | "list_csv" => download_type = DownloadType::List(false),
            "list_json" => download_type = DownloadType::List(true),
            _ => osinfo.push(arg.to_string()),
        }
        args.remove(0);
    };

    if osinfo.len() > 0 {
        if let DownloadType::None = download_type {
            let vm_path = osinfo.iter().map(|s| s.replace(" ", "-") + "-").collect::<String>();
            download_type = DownloadType::Normal(vm_path);
        }
    }

    for _ in osinfo.len()..3 {
        osinfo.push("".into());
    }

    //println!("{:?}", osinfo);

    (osinfo[0].to_lowercase(), osinfo[1].clone(), osinfo[2..].join(" "), download_type, arch.into())
}

enum DownloadType {
    None,
    Normal(String),
    Test,
    Show,
    Homepage,
    List(bool),
}

fn usage(status: i32) {
    std::process::exit(status);
}

fn friendly_urls(url_iso_list: Vec<(String, HeaderMap, String)>) {
    println!("{}", url_iso_list.iter().map(|(url, ..)| url.to_string()).collect::<Vec<_>>().join("\n"));
    std::process::exit(0);
}
