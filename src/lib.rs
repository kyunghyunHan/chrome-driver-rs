use std::{env, fs, io::Cursor, path::Path, process::Command};
use reqwest::blocking::get;
use serde_json::Value;
use zip::ZipArchive;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// ChromeDriver 설치 결과
pub struct DriverInfo {
    /// 크롬드라이버 실행 파일 경로
    pub driver_path: String,
    /// 설치된 버전
    pub version: String,
}

/// 최신 ChromeDriver를 확인/설치
///
/// * 이미 최신 버전이 설치되어 있으면 다운로드를 생략.
/// * macOS(Intel/ARM), Windows 지원.
pub fn ensure_latest_driver(out_dir: &str) -> Result<DriverInfo, Box<dyn std::error::Error>> {
    // 1. 최신 버전 정보
    let versions_url = "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions.json";
    let body = get(versions_url)?.text()?;
    let json: Value = serde_json::from_str(&body)?;
    let version = json["channels"]["Stable"]["version"]
        .as_str()
        .ok_or("Failed to read version")?;
    println!("🌐 Latest ChromeDriver version: {version}");

    // 2. 플랫폼 감지
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

    // 3. 설치 경로
    let driver_path = format!("{}/{}/{}", out_dir, zip_name, exec_name);
    if Path::new(&driver_path).exists() {
        println!("✅ Already installed: {driver_path}");
        return Ok(DriverInfo {
            driver_path,
            version: version.to_string(),
        });
    }

    // 4. 다운로드 URL
    let url = format!(
        "https://edgedl.me.gvt1.com/edgedl/chrome/chrome-for-testing/{}/{} /{}.zip",
        version, platform, zip_name
    ).replace(" /", "/");
    println!("⬇️ Downloading from: {url}");

    // 5. ZIP 다운로드 & 압축 해제
    let resp = get(&url)?;
    let bytes = resp.bytes()?;
    let reader = Cursor::new(bytes);

    fs::create_dir_all(out_dir)?;
    let mut archive = ZipArchive::new(reader)?;
    archive.extract(out_dir)?;

    // 6. 실행 권한 (Unix 전용)
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

/// 설치된 드라이버 버전 확인 (선택)
pub fn check_version(driver_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new(driver_path).arg("--version").status()?;
    println!("Driver check finished with status: {status}");
    Ok(())
}
