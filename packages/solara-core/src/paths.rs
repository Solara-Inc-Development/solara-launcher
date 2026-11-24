//! Solara Launcher directory layout and Maven artifact coordinate utilities.

use anyhow::{Context, bail};
use std::path::{Path, PathBuf};

/// Canonical directory tree owned by `solara-core`.
#[derive(Debug, Clone)]
pub struct SolaraPaths {
    pub root: PathBuf,
}

impl SolaraPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn meta_dir(&self) -> PathBuf {
        self.root.join("meta")
    }

    pub fn versions_dir(&self) -> PathBuf {
        self.meta_dir().join("versions")
    }

    pub fn version_dir(&self, version_id: &str) -> PathBuf {
        self.versions_dir().join(version_id)
    }

    pub fn version_jar(&self, version_id: &str) -> PathBuf {
        self.version_dir(version_id)
            .join(format!("{version_id}.jar"))
    }

    pub fn libraries_dir(&self) -> PathBuf {
        self.meta_dir().join("libraries")
    }

    pub fn assets_dir(&self) -> PathBuf {
        self.meta_dir().join("assets")
    }

    pub fn asset_indexes_dir(&self) -> PathBuf {
        self.assets_dir().join("indexes")
    }

    pub fn asset_objects_dir(&self) -> PathBuf {
        self.assets_dir().join("objects")
    }

    pub fn asset_object_path(&self, sha1_hash: &str) -> anyhow::Result<PathBuf> {
        if sha1_hash.len() < 2 || !sha1_hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            bail!("invalid asset SHA-1 hash: {sha1_hash}");
        }
        Ok(self.asset_objects_dir().join(&sha1_hash[..2]).join(sha1_hash))
    }

    pub fn natives_dir(&self, version_id: &str) -> PathBuf {
        self.meta_dir().join("natives").join(version_id)
    }

    pub fn java_dir(&self) -> PathBuf {
        self.meta_dir().join("java_versions")
    }

    pub fn instances_dir(&self) -> PathBuf {
        self.root.join("profiles")
    }
}

/// Converts a Maven coordinate (`group:artifact:version[:classifier][@ext]`)
/// into a safe relative repository path (e.g. `net/fabricmc/fabric-loader/0.16.9/fabric-loader-0.16.9.jar`).
pub fn maven_coordinate_to_rel_path(coordinate: &str) -> anyhow::Result<String> {
    let trimmed = coordinate.trim();
    if trimmed.is_empty() {
        bail!("empty Maven coordinate");
    }

    let (coord_part, ext) = match trimmed.split_once('@') {
        Some((left, right)) if !right.is_empty() => (left, right),
        Some(_) => bail!("empty extension in Maven coordinate: {coordinate}"),
        None => (trimmed, "jar"),
    };

    let parts: Vec<&str> = coord_part.split(':').collect();
    if parts.len() < 3 || parts.len() > 4 {
        bail!("invalid Maven coordinate '{coordinate}'");
    }

    let group = parts[0];
    let artifact = parts[1];
    let version = parts[2];
    let classifier = parts.get(3).copied();

    for segment in [group, artifact, version, ext]
        .into_iter()
        .chain(classifier.into_iter())
    {
        if segment.is_empty()
            || segment.contains("..")
            || segment.contains('/')
            || segment.contains('\\')
        {
            bail!("unsafe segment '{segment}' in Maven coordinate '{coordinate}'");
        }
    }

    let group_path = group.replace('.', "/");
    let file_name = match classifier {
        Some(cls) => format!("{artifact}-{version}-{cls}.{ext}"),
        None => format!("{artifact}-{version}.{ext}"),
    };

    Ok(format!("{group_path}/{artifact}/{version}/{file_name}"))
}

/// Joins a relative forward-slash path under `base`, rejecting traversal attempts.
pub fn safe_join_rel(base: &Path, rel: &str) -> anyhow::Result<PathBuf> {
    let mut out = base.to_path_buf();
    for part in rel.split(['/', '\\']) {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." || part.contains(':') {
            bail!("path traversal rejected in '{rel}'");
        }
        out.push(part);
    }
    if out == base {
        return Err(anyhow::anyhow!("empty relative path")).context("safe_join_rel");
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_maven_coordinates_and_rejects_traversal() {
        assert_eq!(
            maven_coordinate_to_rel_path("net.fabricmc:fabric-loader:0.16.9").unwrap(),
            "net/fabricmc/fabric-loader/0.16.9/fabric-loader-0.16.9.jar"
        );
        assert_eq!(
            maven_coordinate_to_rel_path("org.lwjgl:lwjgl:3.3.3:natives-windows").unwrap(),
            "org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3-natives-windows.jar"
        );
        assert_eq!(
            maven_coordinate_to_rel_path("com.example:mod:1.0@zip").unwrap(),
            "com/example/mod/1.0/mod-1.0.zip"
        );
        assert!(maven_coordinate_to_rel_path("bad:..:1.0").is_err());
        assert!(safe_join_rel(Path::new("/tmp/root"), "../../etc/passwd").is_err());
    }
}
