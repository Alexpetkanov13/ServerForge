use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tokio::process::Command;

use crate::error::{AppError, AppResult};
use crate::models::{JavaRuntime, JavaTestResult};
use crate::utils::java_version::recommended_java_major;

pub async fn detect_java() -> AppResult<Vec<JavaRuntime>> {
    let mut seen = BTreeSet::new();
    let mut runtimes = Vec::new();

    if let Ok(home) = std::env::var("JAVA_HOME") {
        consider(&home, &mut seen, &mut runtimes).await;
    }
    if let Ok(path) = which::which("java") {
        consider_exe(&path, &mut seen, &mut runtimes).await;
    }
    for dir in common_java_dirs() {
        scan_dir(&dir, &mut seen, &mut runtimes).await;
    }
    #[cfg(windows)]
    scan_registry(&mut seen, &mut runtimes).await;

    runtimes.sort_by(|a, b| b.major.cmp(&a.major).then(a.path.cmp(&b.path)));
    Ok(runtimes)
}

pub async fn pick_java(required_major: u32, preferred: Option<&str>) -> AppResult<JavaRuntime> {
    let runtimes = detect_java().await?;
    if let Some(path) = preferred {
        if let Some(found) = runtimes.iter().find(|r| r.path == path) {
            if found.major >= required_major {
                return Ok(found.clone());
            }
        }
        if let Ok(info) = probe_java(Path::new(path)).await {
            if info.major >= required_major {
                return Ok(info);
            }
        }
    }
    runtimes
        .into_iter()
        .find(|r| r.major >= required_major)
        .ok_or_else(|| {
            AppError::new(
                "Compatible Java not found",
                format!("This server requires Java {required_major}, but none was detected."),
            )
            .with_causes([
                format!("Install Java {required_major} (Eclipse Temurin is recommended)"),
                "Point ServerForge at an existing java.exe from the Java page".to_string(),
            ])
        })
}

pub fn required_java_for(minecraft_version: &str) -> u32 {
    recommended_java_major(minecraft_version)
}

pub async fn test_java(path: &Path) -> AppResult<JavaTestResult> {
    validate_java_executable(path)?;
    let mut cmd = Command::new(path);
    cmd.arg("-version");
    crate::utils::process::capture_hidden(&mut cmd);
    let output = cmd.output().await?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed = parse_java_output(&combined);
    Ok(JavaTestResult {
        ok: output.status.success(),
        version: parsed.0,
        vendor: parsed.1,
        architecture: parsed.2,
        output: combined,
    })
}

pub async fn probe_java(path: &Path) -> AppResult<JavaRuntime> {
    let result = test_java(path).await?;
    if !result.ok {
        return Err(AppError::new(
            "Java test failed",
            "The selected executable did not report a valid Java version.",
        )
        .with_technical(result.output));
    }
    let major = parse_major(&result.version);
    Ok(JavaRuntime {
        id: uuid::Uuid::new_v4().to_string(),
        path: path.to_string_lossy().to_string(),
        version: result.version,
        major,
        vendor: result.vendor,
        architecture: result.architecture,
        is_managed: false,
        compatible: true,
    })
}

pub fn validate_java_executable(path: &Path) -> AppResult<()> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if name != "java" && name != "java.exe" {
        return Err(AppError::new(
            "Invalid Java executable",
            "Please select a java executable, not another program.",
        ));
    }
    if !path.exists() {
        return Err(AppError::new(
            "Java not found",
            "The selected Java path does not exist.",
        ));
    }
    Ok(())
}

pub fn adoptium_download_url(major: u32) -> String {
    let os = if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x64"
    };
    format!(
        "https://api.adoptium.net/v3/binary/latest/{major}/ga/{os}/{arch}/jdk/hotspot/normal/eclipse?project=jdk"
    )
}

async fn consider(home: &str, seen: &mut BTreeSet<String>, out: &mut Vec<JavaRuntime>) {
    let exe = java_bin(Path::new(home));
    consider_exe(&exe, seen, out).await;
}

async fn consider_exe(path: &Path, seen: &mut BTreeSet<String>, out: &mut Vec<JavaRuntime>) {
    let key = path.to_string_lossy().to_ascii_lowercase();
    if !seen.insert(key) {
        return;
    }
    if validate_java_executable(path).is_err() {
        return;
    }
    if let Ok(runtime) = probe_java(path).await {
        out.push(runtime);
    }
}

async fn scan_dir(dir: &Path, seen: &mut BTreeSet<String>, out: &mut Vec<JavaRuntime>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            consider_exe(&java_bin(&path), seen, out).await;
            consider_exe(&java_bin(&path.join("Contents/Home")), seen, out).await;
        }
    }
}

fn java_bin(home: &Path) -> PathBuf {
    if cfg!(windows) {
        home.join("bin").join("java.exe")
    } else {
        home.join("bin").join("java")
    }
}

fn common_java_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if cfg!(windows) {
        for root in [
            std::env::var("ProgramFiles").ok(),
            std::env::var("ProgramFiles(x86)").ok(),
        ]
        .into_iter()
        .flatten()
        {
            let root = PathBuf::from(root);
            dirs.extend([
                root.join("Java"),
                root.join("Eclipse Adoptium"),
                root.join("Microsoft"),
                root.join("Amazon Corretto"),
                root.join("Zulu"),
                root.join("BellSoft"),
                root.join("JavaSoft"),
                root.join("Temurin"),
                root.join("AdoptOpenJDK"),
            ]);
        }
    } else if cfg!(target_os = "macos") {
        dirs.push(PathBuf::from("/Library/Java/JavaVirtualMachines"));
    } else {
        dirs.extend([
            PathBuf::from("/usr/lib/jvm"),
            PathBuf::from("/usr/lib64/jvm"),
        ]);
    }
    dirs
}

#[cfg(windows)]
async fn scan_registry(seen: &mut BTreeSet<String>, out: &mut Vec<JavaRuntime>) {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    for key in [
        r"SOFTWARE\JavaSoft\JDK",
        r"SOFTWARE\JavaSoft\Java Development Kit",
        r"SOFTWARE\JavaSoft\JRE",
        r"SOFTWARE\Eclipse Adoptium",
        r"SOFTWARE\Eclipse Foundation\JDK",
    ] {
        if let Ok(reg) = hklm.open_subkey(key) {
            let homes: Vec<String> = reg
                .enum_keys()
                .flatten()
                .filter_map(|name| {
                    reg.open_subkey(&name)
                        .ok()
                        .and_then(|sub| sub.get_value::<String, _>("JavaHome").ok())
                })
                .collect();
            for home in homes {
                consider(&home, seen, out).await;
            }
        }
    }
}

fn parse_java_output(output: &str) -> (String, String, String) {
    let mut version = String::from("unknown");
    let mut vendor = String::from("unknown");
    let mut arch = String::from("unknown");
    for line in output.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("version") && version == "unknown" {
            if let Some(v) = line.split('"').nth(1) {
                version = v.to_string();
            }
        }
        if lower.contains("64-bit") {
            arch = "x64".into();
        } else if lower.contains("aarch64") || lower.contains("arm64") {
            arch = "aarch64".into();
        } else if lower.contains("32-bit") {
            arch = "x86".into();
        }
        if vendor == "unknown" {
            if lower.contains("temurin") || lower.contains("adoptium") {
                vendor = "Eclipse Adoptium".into();
            } else if lower.contains("hotspot") && lower.contains("openjdk") {
                vendor = "OpenJDK".into();
            } else if lower.contains("zulu") {
                vendor = "Azul Zulu".into();
            } else if lower.contains("corretto") {
                vendor = "Amazon Corretto".into();
            } else if lower.contains("graal") {
                vendor = "GraalVM".into();
            } else if lower.contains("microsoft") {
                vendor = "Microsoft".into();
            } else if lower.contains("oracle") {
                vendor = "Oracle".into();
            }
        }
    }
    if vendor == "unknown" && output.to_ascii_lowercase().contains("openjdk") {
        vendor = "OpenJDK".into();
    }
    (version, vendor, arch)
}

fn parse_major(version: &str) -> u32 {
    if version.starts_with("1.") {
        return version
            .split('.')
            .nth(1)
            .and_then(|s| {
                s.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .ok()
            })
            .unwrap_or(8);
    }
    version
        .split(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modern_and_legacy_versions() {
        assert_eq!(parse_major("21.0.4"), 21);
        assert_eq!(parse_major("17.0.11"), 17);
        assert_eq!(parse_major("1.8.0_402"), 8);
    }
}
