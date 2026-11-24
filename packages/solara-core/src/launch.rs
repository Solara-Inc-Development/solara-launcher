//! Direct Rust JVM argument builder, safe native library extraction, and Minecraft launcher.
//!
//! Launches Minecraft directly via `java`/`javaw` without any inherited Java launch wrapper.

use crate::minecraft::{ArgumentSpec, ArgumentValue, ResolvedLibrary, RuleContext, VersionDetails, rules_allow};
use crate::paths::{SolaraPaths, safe_join_rel};
use anyhow::{Context, bail};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use tokio::process::{Child, Command};

#[derive(Debug, Clone)]
pub struct LaunchConfig {
    pub java_executable: PathBuf,
    pub game_dir: PathBuf,
    pub player_name: String,
    pub player_uuid: String,
    pub access_token: String,
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub extra_jvm_args: Vec<String>,
    pub custom_env: HashMap<String, String>,
    pub resolution: Option<(u32, u32)>,
    pub quick_play_singleplayer: Option<String>,
    pub quick_play_multiplayer: Option<String>,
    pub launcher_version: String,
}

#[derive(Debug, Clone)]
pub struct LaunchCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub env: HashMap<String, String>,
}

impl LaunchCommand {
    /// Returns a log-safe copy of arguments with `--accessToken` redacted.
    pub fn redacted_args(&self) -> Vec<String> {
        let mut out = Vec::with_capacity(self.args.len());
        let mut redact_next = false;
        for arg in &self.args {
            if redact_next {
                out.push("[REDACTED]".to_string());
                redact_next = false;
                continue;
            }
            if arg == "--accessToken" {
                out.push(arg.clone());
                redact_next = true;
            } else {
                out.push(arg.clone());
            }
        }
        out
    }

    pub fn spawn(&self) -> anyhow::Result<Child> {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.args)
            .current_dir(&self.working_dir)
            .envs(&self.env);
        cmd.spawn()
            .with_context(|| format!("failed to spawn Minecraft JVM at {}", self.program.display()))
    }
}

/// Safely extracts native library files (`.dll`, `.so`, `.dylib`) from a ZIP/JAR while blocking ZipSlip.
pub fn extract_native_archive(archive_path: &Path, dest_dir: &Path) -> anyhow::Result<usize> {
    std::fs::create_dir_all(dest_dir)?;
    let file = std::fs::File::open(archive_path)
        .with_context(|| format!("failed to open native archive {}", archive_path.display()))?;
    let mut zip = zip::ZipArchive::new(file)?;
    let mut extracted = 0usize;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let Some(enclosed) = entry.enclosed_name() else {
            bail!("unsafe archive entry path in {}", archive_path.display());
        };
        if enclosed
            .components()
            .next()
            .is_some_and(|c| c.as_os_str() == "META-INF")
        {
            continue;
        }
        let target = dest_dir.join(enclosed);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut out = std::fs::File::create(&target)?;
        io::copy(&mut entry, &mut out)?;
        extracted += 1;
    }

    Ok(extracted)
}

/// Builds the complete direct JVM launch command for a Vanilla, Fabric, or Quilt Minecraft instance.
pub fn build_launch_command(
    paths: &SolaraPaths,
    details: &VersionDetails,
    libraries: &[ResolvedLibrary],
    config: &LaunchConfig,
    base_ctx: &RuleContext,
) -> anyhow::Result<LaunchCommand> {
    let mut ctx = base_ctx.clone();
    if config.resolution.is_some() {
        ctx.features.insert("has_custom_resolution".to_string(), true);
    }
    if config.quick_play_singleplayer.is_some() {
        ctx.features
            .insert("has_quick_plays_support".to_string(), true);
        ctx.features
            .insert("is_quick_play_singleplayer".to_string(), true);
    }
    if config.quick_play_multiplayer.is_some() {
        ctx.features
            .insert("has_quick_plays_support".to_string(), true);
        ctx.features
            .insert("is_quick_play_multiplayer".to_string(), true);
    }

    let cp_sep = if ctx.os_name == "windows" { ";" } else { ":" };
    let mut cp_entries = Vec::new();
    for lib in libraries {
        if !lib.is_native {
            let full = safe_join_rel(&paths.libraries_dir(), &lib.rel_path)?;
            cp_entries.push(full.to_string_lossy().into_owned());
        }
    }
    let client_jar_id = details.inherits_from.as_deref().unwrap_or(&details.id);
    cp_entries.push(
        paths
            .version_jar(client_jar_id)
            .to_string_lossy()
            .into_owned(),
    );
    let classpath = cp_entries.join(cp_sep);

    let natives_dir = paths.natives_dir(&details.id);
    let assets_root = paths.assets_dir();
    let asset_index_id = details
        .asset_index
        .as_ref()
        .map(|a| a.id.as_str())
        .or(details.assets.as_deref())
        .unwrap_or("legacy");

    let mut vars: HashMap<&str, String> = HashMap::from([
        ("auth_player_name", config.player_name.clone()),
        ("version_name", details.id.clone()),
        ("game_directory", config.game_dir.to_string_lossy().into_owned()),
        ("assets_root", assets_root.to_string_lossy().into_owned()),
        ("assets_index_name", asset_index_id.to_string()),
        ("auth_uuid", config.player_uuid.clone()),
        ("auth_access_token", config.access_token.clone()),
        ("user_type", "msa".to_string()),
        (
            "version_type",
            details
                .version_type
                .clone()
                .unwrap_or_else(|| "release".to_string()),
        ),
        ("natives_directory", natives_dir.to_string_lossy().into_owned()),
        ("launcher_name", "Solara Launcher".to_string()),
        ("launcher_version", config.launcher_version.clone()),
        ("classpath", classpath),
        ("classpath_separator", cp_sep.to_string()),
        (
            "library_directory",
            paths.libraries_dir().to_string_lossy().into_owned(),
        ),
    ]);

    if let Some((w, h)) = config.resolution {
        vars.insert("resolution_width", w.to_string());
        vars.insert("resolution_height", h.to_string());
    }
    if let Some(world) = &config.quick_play_singleplayer {
        vars.insert("quickPlaySingleplayer", world.clone());
    }
    if let Some(server) = &config.quick_play_multiplayer {
        vars.insert("quickPlayMultiplayer", server.clone());
    }

    let mut args = Vec::new();
    if config.memory_min_mb > 0 {
        args.push(format!("-Xms{}M", config.memory_min_mb));
    }
    if config.memory_max_mb > 0 {
        args.push(format!("-Xmx{}M", config.memory_max_mb));
    }
    args.extend(config.extra_jvm_args.iter().cloned());

    if let Some(spec) = &details.arguments {
        for item in &spec.jvm {
            expand_argument_spec(item, &ctx, &vars, &mut args);
        }
    } else {
        args.push(format!("-Djava.library.path={}", natives_dir.display()));
        args.push("-cp".to_string());
        args.push(vars["classpath"].clone());
    }

    args.push(details.main_class.clone());

    if let Some(spec) = &details.arguments {
        for item in &spec.game {
            expand_argument_spec(item, &ctx, &vars, &mut args);
        }
    } else if let Some(legacy_game_args) = &details.minecraft_arguments {
        for token in legacy_game_args.split_whitespace() {
            args.push(substitute_vars(token, &vars));
        }
    }

    Ok(LaunchCommand {
        program: config.java_executable.clone(),
        args,
        working_dir: config.game_dir.clone(),
        env: config.custom_env.clone(),
    })
}

fn expand_argument_spec(
    spec: &ArgumentSpec,
    ctx: &RuleContext,
    vars: &HashMap<&str, String>,
    out: &mut Vec<String>,
) {
    match spec {
        ArgumentSpec::Simple(val) => out.push(substitute_vars(val, vars)),
        ArgumentSpec::Conditional { rules, value } => {
            if rules_allow(rules, ctx) {
                match value {
                    ArgumentValue::Single(s) => out.push(substitute_vars(s, vars)),
                    ArgumentValue::Multiple(list) => {
                        for s in list {
                            out.push(substitute_vars(s, vars));
                        }
                    }
                }
            }
        }
    }
}

fn substitute_vars(template: &str, vars: &HashMap<&str, String>) -> String {
    let mut result = template.to_string();
    for (k, v) in vars {
        let needle = format!("${{{k}}}");
        if result.contains(&needle) {
            result = result.replace(&needle, v);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minecraft::{LaunchArguments, VersionDownloads};

    #[test]
    fn builds_direct_jvm_command_and_redacts_access_token() {
        let paths = SolaraPaths::new("/solara");
        let details = VersionDetails {
            id: "1.21.4".to_string(),
            inherits_from: None,
            main_class: "net.minecraft.client.main.Main".to_string(),
            version_type: Some("release".to_string()),
            assets: Some("19".to_string()),
            asset_index: None,
            downloads: VersionDownloads::default(),
            libraries: vec![],
            arguments: Some(LaunchArguments {
                jvm: vec![
                    ArgumentSpec::Simple("-Djava.library.path=${natives_directory}".to_string()),
                    ArgumentSpec::Simple("-cp".to_string()),
                    ArgumentSpec::Simple("${classpath}".to_string()),
                ],
                game: vec![
                    ArgumentSpec::Simple("--username".to_string()),
                    ArgumentSpec::Simple("${auth_player_name}".to_string()),
                    ArgumentSpec::Simple("--accessToken".to_string()),
                    ArgumentSpec::Simple("${auth_access_token}".to_string()),
                ],
            }),
            minecraft_arguments: None,
            java_version: None,
        };
        let libs = vec![ResolvedLibrary {
            coordinate: "com.mojang:logging:1.2.7".to_string(),
            rel_path: "com/mojang/logging/1.2.7/logging-1.2.7.jar".to_string(),
            url: "https://libraries.minecraft.net/com/mojang/logging/1.2.7/logging-1.2.7.jar"
                .to_string(),
            sha1: None,
            is_native: false,
        }];
        let cfg = LaunchConfig {
            java_executable: PathBuf::from("javaw.exe"),
            game_dir: PathBuf::from("/solara/profiles/Survival"),
            player_name: "SolaraPlayer".to_string(),
            player_uuid: "00000000-0000-0000-0000-000000000001".to_string(),
            access_token: "super-secret-msa-token".to_string(),
            memory_min_mb: 1024,
            memory_max_mb: 4096,
            extra_jvm_args: vec!["-XX:+UseG1GC".to_string()],
            custom_env: HashMap::new(),
            resolution: Some((1920, 1080)),
            quick_play_singleplayer: None,
            quick_play_multiplayer: None,
            launcher_version: "0.0.102".to_string(),
        };
        let ctx = RuleContext {
            os_name: "windows".to_string(),
            os_arch: "x86_64".to_string(),
            features: HashMap::new(),
        };

        let cmd = build_launch_command(&paths, &details, &libs, &cfg, &ctx).unwrap();
        assert!(cmd.args.contains(&"-Xmx4096M".to_string()));
        assert!(cmd.args.contains(&"net.minecraft.client.main.Main".to_string()));
        assert!(cmd.args.contains(&"SolaraPlayer".to_string()));

        let safe = cmd.redacted_args();
        assert!(!safe.iter().any(|a| a.contains("super-secret-msa-token")));
        assert!(safe.contains(&"[REDACTED]".to_string()));
    }
}
