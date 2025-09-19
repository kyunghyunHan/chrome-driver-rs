use serde_json::Value;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{env, fs, path::Path};
use tokio::{fs as tokio_fs, io::AsyncWriteExt};
use zip::ZipArchive;
use tokio::task;
/// ChromeDriver 설치 결과
pub struct DriverInfo {
    /// 크롬드라이버 실행 파일 경로
    pub driver_path: String,
    /// 설치된 버전
    pub version: String,
}

/// 최신 ChromeDriver를 확인/설치 (비동기)
///
/// * 이미 최신 버전이 설치되어 있으면 다운로드를 생략.
/// * macOS(Intel/ARM), Windows 지원.
pub async fn ensure_latest_driver(
    out_dir: &str,
) -> Result<DriverInfo, Box<dyn std::error::Error + Send + Sync + 'static>> {
    // 1️⃣ 최신 버전 가져오기
    let versions_url =
        "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions.json";
    let body = reqwest::get(versions_url).await?.text().await?;
    let json: Value = serde_json::from_str(&body)?;
    let version = json["channels"]["Stable"]["version"]
        .as_str()
        .ok_or("Failed to read version")?;
    println!("🌐 Latest ChromeDriver version: {version}");

    // 2️⃣ 플랫폼 감지
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

    // 3️⃣ 설치 경로 확인
    let driver_path = format!("{}/{}/{}", out_dir, zip_name, exec_name);
    if Path::new(&driver_path).exists() {
        println!("✅ Already installed: {driver_path}");
        return Ok(DriverInfo {
            driver_path,
            version: version.to_string(),
        });
    }

    // 4️⃣ 다운로드 URL
    let url = format!(
        "https://edgedl.me.gvt1.com/edgedl/chrome/chrome-for-testing/{}/{}/{}.zip",
        version, platform, zip_name
    );
    println!("⬇️ Downloading from: {url}");

    // 5️⃣ ZIP 다운로드
    let bytes = reqwest::get(&url).await?.bytes().await?;

    // 6️⃣ 압축 해제 (ZipArchive는 동기 → spawn_blocking 사용)
    tokio_fs::create_dir_all(out_dir).await?;
    let out_dir_owned = out_dir.to_owned();
    task::spawn_blocking(
        move || -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
            let reader = std::io::Cursor::new(bytes);
            let mut archive = ZipArchive::new(reader)?;
            archive.extract(&out_dir_owned)?;
            Ok(())
        },
    )
    .await??;

    // 7️⃣ 실행 권한 (유닉스 전용)
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
/// 설치된 드라이버 버전 확인 (비동기)
pub async fn check_version(driver_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let status = tokio::process::Command::new(driver_path)
        .arg("--version")
        .status()
        .await?;
    println!("Driver check finished with status: {status}");
    Ok(())
}
