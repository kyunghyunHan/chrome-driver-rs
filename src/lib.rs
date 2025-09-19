use serde_json::Value;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{env, fs, path::Path};
use tokio::{fs as tokio_fs, task};
use zip::ZipArchive;

/// Information about the installed ChromeDriver
pub struct DriverInfo {
    /// Path to the ChromeDriver executable
    pub driver_path: String,
    /// Installed version
    pub version: String,
}

/// Check and install the latest ChromeDriver asynchronously.
///
/// * If the latest version is already installed, the download is skipped.
/// * Supports macOS (Intel/ARM) and Windows.
pub async fn ensure_latest_driver(
    out_dir: &str,
) -> Result<DriverInfo, Box<dyn std::error::Error + Send + Sync + 'static>> {
    // 1️⃣ Fetch the latest version info
    let versions_url =
        "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions.json";
    let body = reqwest::get(versions_url).await?.text().await?;
    let json: Value = serde_json::from_str(&body)?;
    let version = json["channels"]["Stable"]["version"]
        .as_str()
        .ok_or("Failed to read version")?;
    println!("🌐 Latest ChromeDriver version: {version}");

    // 2️⃣ Detect platform
    let (platform, exec_name, zip_name) = match env::consts::OS {
        "macos" => {
            let arch = env::consts::ARCH;
            if arch == "aarch64" {
                ("mac-arm64", "chromedriver", "chromedriver-mac-arm64")
            } else {
                ("mac-x64", "chromedriver", "chromedriver-mac-x64")
            }
        }
        "windows" => ("win64", "chromedriver.exe", "chromedriver-win64"),
        other => return Err(format!("Unsupported OS: {}", other).into()),
    };

    // 3️⃣ Check if already installed
    let driver_path = format!("{}/{}/{}", out_dir, zip_name, exec_name);
    if Path::new(&driver_path).exists() {
        println!("✅ Already installed: {driver_path}");
        return Ok(DriverInfo {
            driver_path,
            version: version.to_string(),
        });
    }

    // 4️⃣ Build download URL
    let url = format!(
        "https://edgedl.me.gvt1.com/edgedl/chrome/chrome-for-testing/{}/{}/{}.zip",
        version, platform, zip_name
    );
    println!("⬇️ Downloading from: {url}");

    // 5️⃣ Download zip
    let bytes = reqwest::get(&url).await?.bytes().await?;

    // 6️⃣ Extract archive (ZipArchive is blocking → use spawn_blocking)
    tokio_fs::create_dir_all(out_dir).await?;
    let out_dir_owned = out_dir.to_owned();
    task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let reader = std::io::Cursor::new(bytes);
        let mut archive = ZipArchive::new(reader)?;
        archive.extract(&out_dir_owned)?;
        Ok(())
    })
    .await??;

    // 7️⃣ Set execute permissions (Unix only)
    #[cfg(unix)]
    {
        let full_path = Path::new(out_dir).join(zip_name).join(exec_name);
        fs::set_permissions(&full_path, fs::Permissions::from_mode(0o755))?;
    }

    println!("🚀 ChromeDriver ready at: {}", driver_path);

    Ok(DriverInfo {
        driver_path,
        version: version.to_string(),
    })
}

/// Check the installed driver version (async)
pub async fn check_version(driver_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let status = tokio::process::Command::new(driver_path)
        .arg("--version")
        .status()
        .await?;
    println!("Driver check finished with status: {status}");
    Ok(())
}
