//! Independent Java runtime discovery, native `-version` probing, and Adoptium Temurin selection.
//!
//! Replaces the inherited `com.modrinth.theseus.JavaInfo` Java helper with pure Rust probing.

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Discovered Java installation metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JavaRuntime {
    pub path: PathBuf,
    pub version: String,
    pub major_version: u32,
    pub architecture: String,
    pub vendor: Option<String>,
}

/// Returns the recommended Java major version for a given Minecraft release version.
///
/// Matches Mojang's official requirements:
/// - Minecraft 1.20.5+ (and 24w14a+ snapshots) -> Java 21
/// - Minecraft 1.17 ..= 1.20.4 -> Java 17
/// - Minecraft <= 1.16.5 -> Java 8
pub fn recommended_java_major(minecraft_version: &str) -> u32 {
    let clean = minecraft_version.trim();
    let parts: Vec<u32> = clean
        .split(['.', '-', '_'])
        .take(3)
        .filter_map(|p| p.parse::<u32>().ok())
        .collect();

    if parts.len() >= 2 && parts[0] == 1 {
        let minor = parts[1];
        let patch = parts.get(2).copied().unwrap_or(0);
        if minor > 20 || (minor == 20 && patch >= 5) {
            return 21;
        }
        if minor >= 17 {
            return 17;
        }
        return 8;
    }

    // Modern snapshot format like `24w14a` or `25w02a`
    if let Some((year_str, rest)) = clean.split_once('w')
        && let Ok(year) = year_str.parse::<u32>()
    {
        let week: u32 = rest
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap_or(0);
        if year > 24 || (year == 24 && week >= 14) {
            return 21;
        }
        if year >= 21 {
            return 17;
        }
        return 8;
    }

    21
}

/// Official Eclipse Adoptium Temurin binary API v3 redirect URL for auto-installing a JRE.
pub fn adoptium_temurin_binary_url(major: u32, os: &str, arch: &str) -> anyhow::Result<String> {
    let api_os = match os {
        "windows" => "windows",
        "linux" => "linux",
        "macos" | "osx" => "mac",
        other => bail!("unsupported OS for Adoptium JRE: {other}"),
    };
    let api_arch = match arch {
        "x86_64" | "x64" | "amd64" => "x64",
        "aarch64" | "arm64" => "aarch64",
        "x86" | "x32" => "x32",
        other => bail!("unsupported architecture for Adoptium JRE: {other}"),
    };
    Ok(format!(
        "https://api.adoptium.net/v3/binary/latest/{major}/ga/{api_os}/{api_arch}/jre/hotspot/normal/eclipse"
    ))
}

/// Parses `java -XshowSettings:properties -version` output (written to stderr by OpenJDK/Temurin/Oracle).
pub fn parse_java_properties_output(output: &str) -> Option<(String, u32, String, Option<String>)> {
    let mut version: Option<String> = None;
    let mut arch: Option<String> = None;
    let mut vendor: Option<String> = None;

    for raw_line in output.lines() {
        let line = raw_line.trim();
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            let val = v.trim();
            match key {
                "java.version" => version = Some(val.to_string()),
                "os.arch" => arch = Some(normalize_arch(val)),
                "java.vendor" => vendor = Some(val.to_string()),
                _ => {}
            }
        }
    }

    // Fallback: parse `openjdk version "21.0.5"` or `java version "1.8.0_432"`
    if version.is_none() {
        for raw_line in output.lines() {
            if let Some(start) = raw_line.find('"')
                && let Some(end) = raw_line[start + 1..].find('"')
            {
                let candidate = &raw_line[start + 1..start + 1 + end];
                if candidate.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    version = Some(candidate.to_string());
                    break;
                }
            }
        }
    }

    let version = version?;
    let major = parse_major_version(&version)?;
    let arch = arch.unwrap_or_else(|| {
        if output.contains("64-Bit") {
            "x86_64".to_string()
        } else {
            std::env::consts::ARCH.to_string()
        }
    });

    Some((version, major, arch, vendor))
}

pub fn parse_major_version(version: &str) -> Option<u32> {
    let trimmed = version.trim();
    if let Some(rest) = trimmed.strip_prefix("1.") {
        return rest
            .split(['.', '_', '-', '+'])
            .next()
            .and_then(|s| s.parse::<u32>().ok());
    }
    trimmed
        .split(['.', '_', '-', '+'])
        .next()
        .and_then(|s| s.parse::<u32>().ok())
}

fn normalize_arch(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "amd64" | "x86_64" | "x64" => "x86_64".to_string(),
        "aarch64" | "arm64" => "aarch64".to_string(),
        "x86" | "i386" | "i686" => "x86".to_string(),
        other => other.to_string(),
    }
}

/// Executes `<java_path> -XshowSettings:properties -version` directly and returns structured runtime info.
pub async fn probe_java_executable(java_path: &Path) -> anyhow::Result<JavaRuntime> {
    let mut cmd = Command::new(java_path);
    cmd.arg("-XshowSettings:properties").arg("-version");

    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd
        .output()
        .await
        .with_context(|| format!("failed to execute Java binary at {}", java_path.display()))?;

    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let (version, major_version, architecture, vendor) = parse_java_properties_output(&combined)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "could not parse Java version output from {}",
                java_path.display()
            )
        })?;

    Ok(JavaRuntime {
        path: java_path.to_path_buf(),
        version,
        major_version,
        architecture,
        vendor,
    })
}

/// Scans standard OS locations + `JAVA_HOME` + `PATH` + Solara's managed Java folder for candidate binaries.
pub fn discover_candidate_java_paths(solara_java_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = BTreeSet::new();
    let exe_name = if cfg!(windows) { "javaw.exe" } else { "java" };
    let fallback_exe = if cfg!(windows) { "java.exe" } else { "java" };

    let add_home = |set: &mut BTreeSet<PathBuf>, home: PathBuf| {
        let bin_dir = home.join("bin");
        let primary = bin_dir.join(exe_name);
        if primary.is_file() {
            set.insert(primary);
        } else {
            let secondary = bin_dir.join(fallback_exe);
            if secondary.is_file() {
                set.insert(secondary);
            }
        }
    };

    if let Ok(java_home) = std::env::var("JAVA_HOME")
        && !java_home.trim().is_empty()
    {
        add_home(&mut candidates, PathBuf::from(java_home));
    }

    if let Ok(path_env) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_env) {
            for name in [exe_name, fallback_exe] {
                let p = dir.join(name);
                if p.is_file() {
                    candidates.insert(p);
                }
            }
        }
    }

    let mut scan_roots: Vec<PathBuf> = Vec::new();
    if let Some(managed) = solara_java_dir {
        scan_roots.push(managed.to_path_buf());
    }

    #[cfg(target_os = "windows")]
    {
        for env_key in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
            if let Ok(base) = std::env::var(env_key) {
                let root = PathBuf::from(base);
                for sub in [
                    "Eclipse Adoptium",
                    "Java",
                    "Microsoft",
                    "Zulu",
                    "BellSoft",
                    "AdoptOpenJDK",
                ] {
                    scan_roots.push(root.join(sub));
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        scan_roots.push(PathBuf::from("/usr/lib/jvm"));
        scan_roots.push(PathBuf::from("/usr/lib64/jvm"));
    }

    #[cfg(target_os = "macos")]
    {
        scan_roots.push(PathBuf::from("/Library/Java/JavaVirtualMachines"));
    }

    for root in scan_roots {
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    add_home(&mut candidates, p.clone());
                    add_home(&mut candidates, p.join("Contents").join("Home"));
                }
            }
        }
    }

    candidates.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_correct_java_major_for_minecraft_versions() {
        assert_eq!(recommended_java_major("1.21.4"), 21);
        assert_eq!(recommended_java_major("1.20.5"), 21);
        assert_eq!(recommended_java_major("24w14a"), 21);
        assert_eq!(recommended_java_major("1.20.4"), 17);
        assert_eq!(recommended_java_major("1.17.1"), 17);
        assert_eq!(recommended_java_major("1.16.5"), 8);
        assert_eq!(recommended_java_major("1.8.9"), 8);
    }

    #[test]
    fn parses_temurin_and_java8_properties_output() {
        let j21 = r#"
Property settings:
    java.vendor = Eclipse Adoptium
    java.version = 21.0.5
    os.arch = amd64
openjdk version "21.0.5" 2024-10-15 LTS
"#;
        let (ver, major, arch, vendor) = parse_java_properties_output(j21).unwrap();
        assert_eq!(ver, "21.0.5");
        assert_eq!(major, 21);
        assert_eq!(arch, "x86_64");
        assert_eq!(vendor.as_deref(), Some("Eclipse Adoptium"));

        let j8 = r#"
java version "1.8.0_432"
Java(TM) SE Runtime Environment (build 1.8.0_432-b07)
Java HotSpot(TM) 64-Bit Server VM (build 25.432-b07, mixed mode)
"#;
        let (ver8, major8, arch8, _) = parse_java_properties_output(j8).unwrap();
        assert_eq!(ver8, "1.8.0_432");
        assert_eq!(major8, 8);
        assert_eq!(arch8, "x86_64");
    }

    #[test]
    fn builds_official_adoptium_url() {
        let url = adoptium_temurin_binary_url(21, "windows", "x86_64").unwrap();
        assert_eq!(
            url,
            "https://api.adoptium.net/v3/binary/latest/21/ga/windows/x64/jre/hotspot/normal/eclipse"
        );
    }
}
