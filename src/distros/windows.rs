use crate::utils::FormatUrl;
use crate::quickget::handle_download;
use rand::{Rng, thread_rng};
use uuid::Uuid;
use std::error::Error;
use reqwest::header::HeaderMap;
use std::process::Command;

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
        return Err("Microsoft servers gave us an empty response to our request for an automated download.".into());
    } else if download_link_html.contains("We are unable to complete your request at this time.") {
        return Err("Microsoft blocked the automated download request based on your IP address.".into());
    }

    let ending = download_link_html.find(r#""><span class="product-download-type">IsoX64</span:"#).unwrap_or(download_link_html.len());
    let Some(starting) = download_link_html[..ending].rfind("https://software.download.prss.microsoft.com")
        else {
            return Err("Unable to parse download link from HTML.".into());
        };
    let link = download_link_html[starting..ending].into();

    Ok(vec![link])
}

pub fn windows_config(paths: Vec<String>, _: &str, _: &str, _: &str) -> String {
    let path = paths.first().unwrap().split("/").nth(0).unwrap().to_string();
    let drivers: [String; 4] = ["https://fedorapeople.org/groups/virt/virtio-win/direct-downloads/stable-virtio/virtio-win.iso".into(),
        "https://www.spice-space.org/download/windows/spice-webdavd/spice-webdavd-x64-latest.msi".into(), 
        "https://www.spice-space.org/download/windows/vdagent/vdagent-win-0.10.0/spice-vdagent-x64-0.10.0.msi".into(),
        "https://www.spice-space.org/download/windows/usbdk/UsbDk_1.0.22_x64.msi".into()];
    println!("Downloading drivers.");

    std::fs::create_dir(path.clone() + "/unattended").expect("Unable to create unattended directory.");

    let downloads = drivers.into_iter().map(|url| {
        let path = if url.contains("spice-space.org") {
            path.clone() + "/unattended/" + &url.split("/").last().unwrap()
        } else {
            path.clone() + "/" + &url.split("/").last().unwrap()
        };

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                handle_download(url, path, HeaderMap::new()).await
            })
        })
    }).collect::<Vec<_>>();
    downloads.into_iter().for_each(|download| {
        if let Err(e) = download.join().expect("ERROR: Download thread panicked") {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    });
    std::fs::write(path.clone() + "/unattended/autounattend.xml", UNATTENDED_WINDOWS).unwrap();

    match Command::new("mkisofs")
        .arg("-quiet")
        .arg("-l")
        .arg("-o")
        .arg(path.clone() + "/unattended.iso")
        .arg(path.clone() + "/unattended")
        .spawn() {
            Ok(_) => println!("Successfully created unattended setup ISO."),
            Err(e) => eprintln!("Failed to create unattended setup ISO: {}", e),
        };

    
    return format!(r#"fixed_iso="{}""#, path.clone() + "/unattended.iso").into();
}


// Below is the XML for the unattended setup.

const UNATTENDED_WINDOWS: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<unattend xmlns="urn:schemas-microsoft-com:unattend"
  xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <!--
       For documentation on components:
       https://docs.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/
  -->
  <settings pass="offlineServicing">
    <component name="Microsoft-Windows-LUA-Settings" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <EnableLUA>false</EnableLUA>
    </component>
    <component name="Microsoft-Windows-Shell-Setup" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <ComputerName>*</ComputerName>
    </component>
  </settings>

  <settings pass="generalize">
    <component name="Microsoft-Windows-PnPSysprep" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS">
      <PersistAllDeviceInstalls>true</PersistAllDeviceInstalls>
    </component>
    <component name="Microsoft-Windows-Security-SPP" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <SkipRearm>1</SkipRearm>
    </component>
  </settings>

  <settings pass="specialize">
    <component name="Microsoft-Windows-Security-SPP-UX" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <SkipAutoActivation>true</SkipAutoActivation>
    </component>
    <component name="Microsoft-Windows-Shell-Setup" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <ComputerName>*</ComputerName>
      <OEMInformation>
        <Manufacturer>Quickemu Project</Manufacturer>
        <Model>Quickemu</Model>
        <SupportHours>24/7</SupportHours>
        <SupportPhone></SupportPhone>
        <SupportProvider>Quickemu Project</SupportProvider>
        <SupportURL>https://github.com/quickemu-project/quickemu/issues</SupportURL>
      </OEMInformation>
      <OEMName>Quickemu Project</OEMName>
      <ProductKey>W269N-WFGWX-YVC9B-4J6C9-T83GX</ProductKey>
    </component>
    <component name="Microsoft-Windows-SQMApi" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <CEIPEnabled>0</CEIPEnabled>
    </component>
  </settings>

  <settings pass="windowsPE">
    <component name="Microsoft-Windows-Setup" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <Diagnostics>
        <OptIn>false</OptIn>
      </Diagnostics>
      <DiskConfiguration>
        <Disk wcm:action="add">
          <DiskID>0</DiskID>
          <WillWipeDisk>true</WillWipeDisk>
          <CreatePartitions>
            <!-- Windows RE Tools partition -->
            <CreatePartition wcm:action="add">
              <Order>1</Order>
              <Type>Primary</Type>
              <Size>256</Size>
            </CreatePartition>
            <!-- System partition (ESP) -->
            <CreatePartition wcm:action="add">
              <Order>2</Order>
              <Type>EFI</Type>
              <Size>128</Size>
            </CreatePartition>
            <!-- Microsoft reserved partition (MSR) -->
            <CreatePartition wcm:action="add">
              <Order>3</Order>
              <Type>MSR</Type>
              <Size>128</Size>
            </CreatePartition>
            <!-- Windows partition -->
            <CreatePartition wcm:action="add">
              <Order>4</Order>
              <Type>Primary</Type>
              <Extend>true</Extend>
            </CreatePartition>
          </CreatePartitions>
          <ModifyPartitions>
            <!-- Windows RE Tools partition -->
            <ModifyPartition wcm:action="add">
              <Order>1</Order>
              <PartitionID>1</PartitionID>
              <Label>WINRE</Label>
              <Format>NTFS</Format>
              <TypeID>DE94BBA4-06D1-4D40-A16A-BFD50179D6AC</TypeID>
            </ModifyPartition>
            <!-- System partition (ESP) -->
            <ModifyPartition wcm:action="add">
              <Order>2</Order>
              <PartitionID>2</PartitionID>
              <Label>System</Label>
              <Format>FAT32</Format>
            </ModifyPartition>
            <!-- MSR partition does not need to be modified -->
            <ModifyPartition wcm:action="add">
              <Order>3</Order>
              <PartitionID>3</PartitionID>
            </ModifyPartition>
            <!-- Windows partition -->
              <ModifyPartition wcm:action="add">
              <Order>4</Order>
              <PartitionID>4</PartitionID>
              <Label>Windows</Label>
              <Letter>C</Letter>
              <Format>NTFS</Format>
            </ModifyPartition>
          </ModifyPartitions>
        </Disk>
      </DiskConfiguration>
      <DynamicUpdate>
        <Enable>true</Enable>
        <WillShowUI>Never</WillShowUI>
      </DynamicUpdate>
      <ImageInstall>
        <OSImage>
          <InstallTo>
            <DiskID>0</DiskID>
            <PartitionID>4</PartitionID>
          </InstallTo>
          <InstallToAvailablePartition>false</InstallToAvailablePartition>
        </OSImage>
      </ImageInstall>
      <RunSynchronous>
        <RunSynchronousCommand wcm:action="add">
          <Order>1</Order>
          <Path>reg add HKLM\System\Setup\LabConfig /v BypassCPUCheck /t REG_DWORD /d 0x00000001 /f</Path>
        </RunSynchronousCommand>
        <RunSynchronousCommand wcm:action="add">
          <Order>2</Order>
          <Path>reg add HKLM\System\Setup\LabConfig /v BypassRAMCheck /t REG_DWORD /d 0x00000001 /f</Path>
        </RunSynchronousCommand>
        <RunSynchronousCommand wcm:action="add">
          <Order>3</Order>
          <Path>reg add HKLM\System\Setup\LabConfig /v BypassSecureBootCheck /t REG_DWORD /d 0x00000001 /f</Path>
        </RunSynchronousCommand>
        <RunSynchronousCommand wcm:action="add">
          <Order>4</Order>
          <Path>reg add HKLM\System\Setup\LabConfig /v BypassTPMCheck /t REG_DWORD /d 0x00000001 /f</Path>
        </RunSynchronousCommand>
      </RunSynchronous>
      <UpgradeData>
        <Upgrade>false</Upgrade>
        <WillShowUI>Never</WillShowUI>
      </UpgradeData>
      <UserData>
        <AcceptEula>true</AcceptEula>
        <FullName>Quickemu</FullName>
        <Organization>Quickemu Project</Organization>
        <!-- https://docs.microsoft.com/en-us/windows-server/get-started/kms-client-activation-keys -->
        <ProductKey>
          <Key>W269N-WFGWX-YVC9B-4J6C9-T83GX</Key>
          <WillShowUI>Never</WillShowUI>
        </ProductKey>
      </UserData>
    </component>

    <component name="Microsoft-Windows-PnpCustomizationsWinPE" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS" processorArchitecture="amd64" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <!--
           This makes the VirtIO drivers available to Windows, assuming that
           the VirtIO driver disk is available as drive E:
           https://github.com/virtio-win/virtio-win-pkg-scripts/blob/master/README.md
      -->
      <DriverPaths>
        <PathAndCredentials wcm:action="add" wcm:keyValue="1">
          <Path>E:\qemufwcfg\w10\amd64</Path>
        </PathAndCredentials>
        <PathAndCredentials wcm:action="add" wcm:keyValue="2">
          <Path>E:\vioinput\w10\amd64</Path>
        </PathAndCredentials>
        <PathAndCredentials wcm:action="add" wcm:keyValue="3">
          <Path>E:\vioscsi\w10\amd64</Path>
        </PathAndCredentials>
        <PathAndCredentials wcm:action="add" wcm:keyValue="4">
          <Path>E:\viostor\w10\amd64</Path>
        </PathAndCredentials>
        <PathAndCredentials wcm:action="add" wcm:keyValue="5">
          <Path>E:\vioserial\w10\amd64</Path>
        </PathAndCredentials>
        <PathAndCredentials wcm:action="add" wcm:keyValue="6">
          <Path>E:\qxldod\w10\amd64</Path>
        </PathAndCredentials>
        <PathAndCredentials wcm:action="add" wcm:keyValue="7">
          <Path>E:\amd64\w10</Path>
        </PathAndCredentials>
        <PathAndCredentials wcm:action="add" wcm:keyValue="8">
          <Path>E:\viogpudo\w10\amd64</Path>
        </PathAndCredentials>
        <PathAndCredentials wcm:action="add" wcm:keyValue="9">
          <Path>E:\viorng\w10\amd64</Path>
        </PathAndCredentials>
        <PathAndCredentials wcm:action="add" wcm:keyValue="10">
          <Path>E:\NetKVM\w10\amd64</Path>
        </PathAndCredentials>
        <PathAndCredentials wcm:action="add" wcm:keyValue="11">
          <Path>E:\viofs\w10\amd64</Path>
        </PathAndCredentials>
        <PathAndCredentials wcm:action="add" wcm:keyValue="12">
          <Path>E:\Balloon\w10\amd64</Path>
        </PathAndCredentials>
      </DriverPaths>
    </component>
  </settings>

  <settings pass="oobeSystem">
    <component name="Microsoft-Windows-Shell-Setup" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <AutoLogon>
        <Password>
          <Value>quickemu</Value>
          <PlainText>true</PlainText>
        </Password>
        <Enabled>true</Enabled>
        <Username>Quickemu</Username>
      </AutoLogon>
      <DisableAutoDaylightTimeSet>false</DisableAutoDaylightTimeSet>
      <OOBE>
        <HideEULAPage>true</HideEULAPage>
        <HideLocalAccountScreen>true</HideLocalAccountScreen>
        <HideOEMRegistrationScreen>true</HideOEMRegistrationScreen>
        <HideOnlineAccountScreens>true</HideOnlineAccountScreens>
        <HideWirelessSetupInOOBE>true</HideWirelessSetupInOOBE>
        <NetworkLocation>Home</NetworkLocation>
        <ProtectYourPC>3</ProtectYourPC>
        <SkipUserOOBE>true</SkipUserOOBE>
        <SkipMachineOOBE>true</SkipMachineOOBE>
        <VMModeOptimizations>
          <SkipWinREInitialization>true</SkipWinREInitialization>
        </VMModeOptimizations>
      </OOBE>
      <UserAccounts>
        <LocalAccounts>
          <LocalAccount wcm:action="add">
            <Password>
              <Value>quickemu</Value>
              <PlainText>true</PlainText>
            </Password>
            <Description>Quickemu</Description>
            <DisplayName>Quickemu</DisplayName>
            <Group>Administrators</Group>
            <Name>Quickemu</Name>
          </LocalAccount>
        </LocalAccounts>
      </UserAccounts>
      <RegisteredOrganization>Quickemu Project</RegisteredOrganization>
      <RegisteredOwner>Quickemu</RegisteredOwner>
      <FirstLogonCommands>
        <SynchronousCommand wcm:action="add">
          <CommandLine>msiexec /i E:\guest-agent\qemu-ga-x86_64.msi /quiet /passive /qn</CommandLine>
          <Description>Install Virtio Guest Agent</Description>
          <Order>1</Order>
        </SynchronousCommand>
        <SynchronousCommand wcm:action="add">
          <CommandLine>msiexec /i F:\spice-webdavd-x64-latest.msi /quiet /passive /qn</CommandLine>
          <Description>Install spice-webdavd file sharing agent</Description>
          <Order>2</Order>
        </SynchronousCommand>
        <SynchronousCommand wcm:action="add">
          <CommandLine>msiexec /i F:\UsbDk_1.0.22_x64.msi /quiet /passive /qn</CommandLine>
          <Description>Install usbdk USB sharing agent</Description>
          <Order>3</Order>
        </SynchronousCommand>
        <SynchronousCommand wcm:action="add">
          <CommandLine>msiexec /i F:\spice-vdagent-x64-0.10.0.msi /quiet /passive /qn</CommandLine>
          <Description>Install spice-vdagent SPICE agent</Description>
          <Order>4</Order>
        </SynchronousCommand>
        <SynchronousCommand wcm:action="add">
          <CommandLine>Cmd /c POWERCFG -H OFF</CommandLine>
          <Description>Disable Hibernation</Description>
          <Order>5</Order>
        </SynchronousCommand>
      </FirstLogonCommands>
    </component>
  </settings>
</unattend>"#;
