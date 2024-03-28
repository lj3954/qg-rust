use rand::seq::SliceRandom;
use std::error::Error;
use reqwest::header::{self, HeaderMap};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const TYPE_SID: usize = 16;
const TYPE_K: usize = 64;
const TYPE_FG: usize = 64;

const INFO_IMAGE_LINK: &str = "AU";
const INFO_IMAGE_SESS: &str = "AT";
const INFO_SIGN_LINK: &str = "CU";
const INFO_SIGN_SESS: &str = "CT";


pub fn get_urls(release: &str, _: &str, _: &str) -> Result<Vec<(String, HeaderMap)>, Box<dyn Error>> { 
    let (board_id, mlb) = match release {
        "high-sierra" => ("Mac-BE088AF8C5EB4FA2", "00000000000J80300"),
        "mojave" => ("Mac-7BA5B2DFE22DDD8C", "00000000000KXPG00"),
        "catalina" => ("Mac-00BE6ED71E35EB86", "00000000000000000"),
        "big-sur" => ("Mac-42FD25EABCABB274", "00000000000000000"),
        "monterey" => ("Mac-E43C1C25D4880AD6", "00000000000000000"),
        "ventura" => ("Mac-BE088AF8C5EB4FA2", "00000000000000000"),
        "sonoma" => ("Mac-53FDB3D8DB8CA971", "00000000000000000"),
        _ => return Err("Invalid release".into())
    };

    let generate_id = |chars: usize| -> String {
        let characters = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];
        (0..chars).into_iter().map(|_| {
            characters.choose(&mut rand::thread_rng()).unwrap()
        }).collect::<String>()
    };
        
    let post = [
        ("cid", generate_id(TYPE_SID)),
        ("sn", mlb.into()),
        ("bid", board_id.into()),
        ("k", generate_id(TYPE_K)),
        ("fg", generate_id(TYPE_FG)),
        ("os", "default".into()),
    ];

    let reqwest = reqwest::blocking::Client::new();

    // Get session cookie, which reqwest will store.
    let session_request = reqwest.get("http://osrecovery.apple.com/")
        .header(header::HOST, "osrecovery.apple.com")
        .header(header::USER_AGENT, "InternetRecovery/1.0")
        .send()?;
    let session_cookie = session_request.cookies().nth(0).unwrap();

    // Send POST request to get necessary information
    let info = reqwest.post("http://osrecovery.apple.com/InstallationPayload/RecoveryImage")
        .header(header::HOST, "osrecovery.apple.com")
        .header(header::CONNECTION, "close")
        .header(header::USER_AGENT, "InternetRecovery/1.0")
        .header(header::CONTENT_TYPE, "text/plain")
        .header(reqwest::header::COOKIE, format!("{}={}", session_cookie.name(), session_cookie.value()))
        .body(post.iter().map(|(key, value)| format!("\n{}={}", key, value)).collect::<Vec<String>>().join(""))
        .send()?
        .text()?;

    let mut info = info.lines().map(|line| {
        line.split_once(": ").unwrap_or(("",""))
    });

    let headers = |cookie: String| { 
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, "oscdn.apple.com".parse().unwrap());
        headers.insert(header::CONNECTION, "close".parse().unwrap());
        headers.insert(header::USER_AGENT, "InternetRecovery/1.0".parse().unwrap());
        headers.insert(header::COOKIE, cookie.parse().unwrap());
        headers
    };

    let image_link = info.find(|(key, _)| key == &INFO_IMAGE_LINK).unwrap().1;
    let image_headers = headers(format!("AssetToken={}", info.find(|(key, _)| key == &INFO_IMAGE_SESS).unwrap().1));

    let chunklist_link = info.find(|(key, _)| key == &INFO_SIGN_LINK).unwrap().1;
    let chunklist_headers = headers(format!("AssetToken={}", info.find(|(key, _)| key == &INFO_SIGN_SESS).unwrap().1));
        
    println!("{}\n{:?}\n{}\n{:?}", image_link, image_headers, chunklist_link, chunklist_headers);
    Ok(vec![(image_link.to_string(), image_headers), (chunklist_link.to_string(), chunklist_headers)])
}

pub fn macos_config(_: Vec<String>, release: &str, _: &str, _: &str) -> String {
    format!("macos_release={}{}", release, if release == "monterey" { "\ncpu_cores=2" } else { "" })
}

pub fn verify_chunklist(paths: &Vec<String>, _: &str, _: &str, _: &str) -> bool {
    let mut chunklist = File::open(&paths[1]).unwrap();
    let mut buf = vec![0; 36];
    chunklist.read_exact(&mut buf).unwrap();

    let header = ChunklistHdr {
        cl_magic: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
        _cl_header_size: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
        cl_file_ver: buf[8],
        cl_chunk_method: buf[9],
        cl_sig_method: buf[10],
        _unused: buf[11],
        cl_chunk_count: u64::from_le_bytes(buf[12..20].try_into().unwrap()),
        cl_chunk_offset: u64::from_le_bytes(buf[20..28].try_into().unwrap()),
        _cl_sig_offset: u64::from_le_bytes(buf[28..36].try_into().unwrap()),
    };

    if header.cl_magic != CHUNKLIST_MAGIC || header.cl_file_ver != CHUNKLIST_FILE_VERSION_10 || header.cl_sig_method != CHUNKLIST_SIGNATURE_METHOD_10 || header.cl_chunk_method != CHUNKLIST_CHUNK_METHOD_10 {
        return false;
    }

    chunklist.seek(SeekFrom::Start(header.cl_chunk_offset)).unwrap();

    let mut dmg = File::open(&paths[0]).unwrap();
    (0..header.cl_chunk_count).all(|_| {
        let mut buf = [0; 0x24];
        chunklist.read_exact(&mut buf).unwrap();
        let chunk = ChunklistChunk {
            chunk_size: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            chunk_sha256: buf[4..36].try_into().unwrap(),
        };
        
        let mut data = vec![0; chunk.chunk_size as usize];
        dmg.read_exact(&mut data).unwrap();

        let digest = Sha256::digest(&data);
        digest[0..SHA256_DIGEST_LEN as usize] == chunk.chunk_sha256[0..SHA256_DIGEST_LEN as usize]
    })
}

const CHUNKLIST_MAGIC: u32 = 0x4C4B4E43;
const CHUNKLIST_FILE_VERSION_10: u8 = 1;
const CHUNKLIST_CHUNK_METHOD_10: u8 = 1;
const CHUNKLIST_SIGNATURE_METHOD_10: u8 = 1;
const SHA256_DIGEST_LEN: u32 = 32;

#[derive(Debug)]
struct ChunklistHdr {
    cl_magic: u32,
    _cl_header_size: u32,
    cl_file_ver: u8,
    cl_chunk_method: u8,
    cl_sig_method: u8,
    _unused: u8,
    cl_chunk_count: u64,
    cl_chunk_offset: u64,
    _cl_sig_offset: u64,
}
#[derive(Debug)]
struct ChunklistChunk {
    chunk_size: u32,
    chunk_sha256: [u8; 32],
}
