# chrome-driver-rs

A lightweight **Rust library** to automatically download, install, and verify the latest
[ChromeDriver](https://chromedriver.chromium.org/) for macOS (Intel/ARM) and Windows.

This library is especially useful when you need a **ready-to-use** ChromeDriver for
projects using [Selenium](https://www.selenium.dev/) or
[thirtyfour](https://crates.io/crates/thirtyfour) without manual installation.

---

## ✨ Features
- 🔄 Automatically fetches the **latest stable version** of ChromeDriver.
- 💻 Supports:
  - **macOS (Intel/x64 and Apple Silicon ARM64)**
  - **Windows 64-bit**
- ⚡ Async API using **Tokio**.
- 🚀 Automatically sets executable permissions on Unix systems.

---

## 📦 Installation

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
chrome-driver-rs = "0.1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
zip = "0.6"
serde_json = "1"
```

---

## 🚀 Quick Start

```rust
use chrome_driver_rs::{ensure_latest_driver, check_version};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Download and install the latest ChromeDriver
    let driver_info = ensure_latest_driver("./driver").await?;
    println!("ChromeDriver path: {}", driver_info.driver_path);
    println!("Installed version: {}", driver_info.version);

    // (Optional) Verify that the driver is working
    check_version(&driver_info.driver_path).await?;

    Ok(())
}
```

This will:
1. Download the latest **stable** ChromeDriver.
2. Extract it into the `./driver` folder.
3. Print the executable path and version.

---

## 🔧 Example Integration with `thirtyfour`

```rust
use chrome_driver_rs::ensure_latest_driver;
use thirtyfour::prelude::*;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let driver = ensure_latest_driver("./driver").await?;
    let caps = DesiredCapabilities::chrome();
    let webdriver = WebDriver::new("http://localhost:9515", caps).await?;

    webdriver.goto("https://www.google.com").await?;
    println!("Chrome opened successfully!");
    webdriver.quit().await?;

    Ok(())
}
```

---

## 📜 License
Licensed under either of
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
at your option.
