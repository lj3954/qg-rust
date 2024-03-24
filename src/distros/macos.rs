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


pub fn get_urls(release: &str, edition: &str, arch: &str) -> Result<Vec<(String, HeaderMap)>, Box<dyn Error>> { 
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

pub fn verify_chunklist(_: &Vec<String>, _: &str, _: &str, _: &str) -> bool {
    todo!()
}
// WIP image verification below.

//pub fn verify_chunklist(paths: &Vec<String>, _: &str, _: &str, _: &str) -> bool {
//    let mut dmg = File::open(&paths[0]).unwrap();
//    for chunk in parse_chunklist(paths[1]) {
//        dmg.seek(SeekFrom::Current(chunk.chunk_size as i64)).unwrap();
//        let mut buf = vec![0; chunk.chunk_size as usize];
//        dmg.read_exact(&mut buf).unwrap();
//        let hash = Sha256::digest(&<Vec<u8> as TryInto<[u8; usize]>>::try_into(buf).unwrap());
//        if hash.as_slice() != chunk.chunk_sha256 {
//            return false;
//        }
//    }
//    true
//}
//
//const EFI_KEY: &str = "0xC3E748CAD9CD384329E10E25A91E43E1A762FF529ADE578C935BDDF9B13F2179D4855E6FC89E9E29CA12517D17DFA1EDCE0BEBF0EA7B461FFE61D94E2BDF72C196F89ACD3536B644064014DAE25A15DB6BB0852ECBD120916318D1CCDEA3C84C92ED743FC176D0BACA920D3FCF3158AFF731F88CE0623182A8ED67E650515F75745909F07D415F55FC15A35654D118C55A462D37A3ACDA08612F3F3F6571761EFCCBCC299AEE99B3A4FD6212CCFFF5EF37A2C334E871191F7E1C31960E010A54E86FA3F62E6D6905E1CD57732410A3EB0C6B4DEFDABE9F59BF1618758C751CD56CEF851D1C0EAA1C558E37AC108DA9089863D20E2E7E4BF475EC66FE6B3EFDCF";
//const CHUNKLIST_MAGIC: u32 = 0x4C4B4E43;
//const CHUNKLIST_FILE_VERSION_10: u8 = 1;
//const CHUNKLIST_CHUNK_METHOD_10: u8 = 1;
//const CHUNKLIST_SIGNATURE_METHOD_10: u8 = 1;
//const SHA256_DIGEST_LEN: u32 = 32;
//
//
//struct ChunklistHdr {
//    cl_magic: u32,
//    cl_header_size: u32,
//    cl_file_ver: u8,
//    cl_chunk_method: u8,
//    cl_sig_method: u8,
//    _unused: u8,
//    cl_chunk_count: u64,
//    cl_chunk_offset: u64,
//    cl_sig_offset: u64,
//}
//
//struct ChunklistChunk {
//    chunk_size: u32,
//    chunk_sha256: [u8; 32],
//}
//
//fn parse_chunklist(path: String) -> Vec<ChunklistChunk> {
//    let mut file = File::open(path).unwrap();
//    let mut hash = Sha256::new();
//
//    let chunklist_size = std::mem::size_of::<ChunklistHdr>();
//    assert!(chunklist_size == 0x24);
//
//    file.seek(SeekFrom::Start(chunklist_size as u64)).unwrap();
//
//    let mut buf = vec![0; chunklist_size];
//    file.read_exact(&mut buf);
//
//    let header = ChunklistHdr {
//        cl_magic: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
//        cl_header_size: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
//        cl_file_ver: buf[8],
//        cl_chunk_method: buf[9],
//        cl_sig_method: buf[10],
//        _unused: buf[11],
//        cl_chunk_count: u64::from_le_bytes(buf[12..20].try_into().unwrap()),
//        cl_chunk_offset: u64::from_le_bytes(buf[20..28].try_into().unwrap()),
//        cl_sig_offset: u64::from_le_bytes(buf[28..36].try_into().unwrap()),
//    };
//
//    assert_eq!(header.cl_magic, CHUNKLIST_MAGIC);
//    assert_eq!(header.cl_file_ver, CHUNKLIST_FILE_VERSION_10);
//    assert_eq!(header.cl_chunk_method, CHUNKLIST_CHUNK_METHOD_10);
//    assert_eq!(header.cl_sig_method, CHUNKLIST_SIGNATURE_METHOD_10);
//    assert!(header.cl_sig_method == 0 || header.cl_sig_method == 1);
//    assert!(header.cl_chunk_count > 0);
//    assert!(header.cl_chunk_offset == 0x24);
//    assert_eq!(header.cl_sig_offset, header.cl_chunk_offset + header.cl_chunk_count * 0x24);
//
//    let chunks: Vec<ChunklistChunk> = (0..header.cl_chunk_count).map(|_| {
//        let mut buf = vec![0; 0x24];
//        file.seek(SeekFrom::Current(0x24)).unwrap();
//        file.read_exact(&mut buf);
//        hash.update(&buf);
//        ChunklistChunk {
//            chunk_size: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
//            chunk_sha256: hash.clone().finalize().into(),
//        }
//    }).collect();
//
//    chunks
//}
//
