use crate::distros::ErrorKind;
use crate::utils::FormatUrl;
use rand::{Rng, thread_rng};
use uuid::Uuid;
use std::error::Error;

pub fn get_windows_url(release: &str, edition: &str, arch: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let url = match release {
        "8"|"10" => "https://www.microsoft.com/en-us/software-download/windows{RELEASE}ISO",
        _ => "https://www.microsoft.com/en-us/software-download/windows{RELEASE}",
    }.format(release, edition, arch);

    let firefox_release = thread_rng().gen_range(110..=124);
    let useragent = format!("Mozilla/5.0 (X11; Linux x86_64; rv:{}.0) Gecko/20100101 Firefox/{}.0", firefox_release, firefox_release);
    let sessionid = Uuid::new_v4();

    let reqwest = reqwest::blocking::Client::new();

    let mut download_page_html = reqwest.get(&url)
        .header(reqwest::header::USER_AGENT, &useragent)
        .header(reqwest::header::ACCEPT, "")
        .send().map_err(|e| format!("{} while trying to send a request to the download page.", e))?
        .text()?;
    download_page_html.truncate(102400);

    let product_id = download_page_html.split("option").find_map(|value| {
        if value.contains("value=\"") && value.contains(">Windows") {
            let start = value.find("value=\"").unwrap() + 7;
            let end = value.find("\">Windows").unwrap();
            return Some(value.get(start..end).unwrap());
        }
        None
    }).unwrap();

    reqwest.get(format!("https://vlscppe.microsoft.com/tags?org_id=y6jn8c31&session_id={}", sessionid))
        .header(reqwest::header::ACCEPT, "")
        .header(reqwest::header::USER_AGENT, &useragent)
        .send()?;

    let url_segment = &url.split("/").last().unwrap();

    let mut skuid_table = reqwest.post(format!("https://www.microsoft.com/en-US/api/controls/contentinclude/html?pageId=a8f8f489-4c7f-463a-9ca6-5cff94d8d041&host=www.microsoft.com&segments=software-download,{}&query=&action=getskuinformationbyproductedition&sessionId={}&productEditionId={}&sdVersion=2", url_segment, sessionid, product_id))
        .header(reqwest::header::USER_AGENT, &useragent)
        .header(reqwest::header::ACCEPT, "")
        .header(reqwest::header::REFERER, &url)
        .body("")
        .send().map_err(|e| format!("{} while trying to find the available SKUs.", e))?
        .text()?;
    skuid_table.truncate(10240);

    let skuid = skuid_table.lines().find(|line| line.contains(edition))
        .unwrap()
        .split("&quot;").nth(3).unwrap();

    let mut download_link_html = reqwest.post(format!("https://www.microsoft.com/en-US/api/controls/contentinclude/html?pageId=6e2a1789-ef16-4f27-a296-74ef7ef5d96b&host=www.microsoft.com&segments=software-download,{}&query=&action=GetProductDownloadLinksBySku&sessionId={}&skuId={}&language=English&sdVersion=2", url_segment, sessionid, skuid))
        .header(reqwest::header::USER_AGENT, &useragent)
        .header(reqwest::header::ACCEPT, "")
        .header(reqwest::header::REFERER, &url)
        .body("")
        .send().map_err(|e| format!("{} while trying to find the download link.", e))?
        .text()?;
    download_link_html.truncate(4096);

    if download_link_html.is_empty() {
        return Err(Box::new(std::io::Error::new(ErrorKind::Other, "Microsoft servers gave us an empty response to our request for an automated download.")));
    } else if download_link_html.contains("We are unable to complete your request at this time.") {
        return Err(Box::new(std::io::Error::new(ErrorKind::Other, "Microsoft blocked the automated download request based on your IP address.")));
    }

    let starting = download_link_html.rfind("https://software.download.prss.microsoft.com").expect("Unable toparse download link.");
    let ending = download_link_html.find(r#""><span class="product-download-type">IsoX64</span:"#).unwrap_or(download_link_html.len());
    let link = download_link_html[starting..ending].into();
    let drivers = "https://fedorapeople.org/groups/virt/virtio-win/direct-downloads/stable-virtio/virtio-win.iso".into();
    let unattended1 = "https://www.spice-space.org/download/windows/spice-webdavd/spice-webdavd-x64-latest.msi".into();
    let unattended2 = "https://www.spice-space.org/download/windows/vdagent/vdagent-win-0.10.0/spice-vdagent-x64-0.10.0.msi".into();
    let unattended3 = "https://www.spice-space.org/download/windows/usbdk/UsbDk_1.0.22_x64.msi".into();

    Ok(vec![link, drivers, unattended1, unattended2, unattended3])
}

pub fn windows_config(_: &str, _: &str, _: &str) -> String {
    return "#PLACEHOLDER".into();
}
