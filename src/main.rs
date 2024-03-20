mod utils;
mod distros;

use reqwest::header::HeaderMap;
use utils::{Distro, handle_download, Validation, verify_image};


fn main() {
//    let distros = collect_distros().unwrap_or_else(|e| {
//        eprintln!("ERROR: {}", e);
//        std::process::exit(1);
//    });

    let distros = distros::distros();
    let (os, release, edition) = get_args();
    let distro = distros.validate_parameters(&os, &release, &edition);

    println!("{:?}", distro);


   println!("OS: {}, Release: {}, Edition: {}", os, release, edition);

   let url_iso_list = distro.get_url_iso(&release, &edition, "amd64");
    let vm_path = match edition.as_str() {
        "" => format!("{}-{}", os, release),
        _ => format!("{}-{}-{}", os, release, edition),
    };
    std::fs::create_dir(&vm_path).unwrap_or(());
    println!("Downloading to {}", &vm_path);
    spawn_downloads(url_iso_list, vm_path, distro, &release, &edition, "amd64");
}

fn get_args() -> (String, String, String) {
    let blank = String::from("");
    let args: Vec<String> = std::env::args().collect();
    let os = args.get(1).unwrap_or(&blank);
    let release = args.get(2).unwrap_or(&blank);
    let edition = args.get(3).unwrap_or(&blank);
   (os.to_lowercase(), release.to_string(), edition.to_string())
}

fn spawn_downloads(url_iso_list: Vec<(String, HeaderMap, String)>, vm_path: String, distro: Distro, release: &str, edition: &str, arch: &str) {
    for (url, headers, iso) in url_iso_list {
        let path = vm_path.clone() + "/" +  iso.as_str();
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
