use super::{java, launch, plan};
use crate::game_core::auth::{Profile, Session};
use crate::game_core::metadata::Artifact;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn plan_filters_non_x64_native_classifiers_even_when_arch_is_omitted() {
    let version = json!({
        "id": "26.3",
        "type": "release",
        "mainClass": "net.minecraft.client.main.Main",
        "javaVersion": { "majorVersion": 25 },
        "downloads": {
            "client": {
                "url": "https://piston-data.mojang.com/v1/objects/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/client.jar",
                "sha1": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "size": 100
            }
        },
        "assetIndex": {
            "id": "26",
            "url": "https://piston-meta.mojang.com/v1/packages/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb/26.json",
            "sha1": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "size": 50
        },
        "libraries": [
            {
                "name": "org.lwjgl:lwjgl:3.4.3+4",
                "downloads": { "artifact": {
                    "path": "org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4.jar",
                    "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4.jar",
                    "sha1": "1111111111111111111111111111111111111111",
                    "size": 10
                }}
            },
            {
                "name": "org.lwjgl:lwjgl:3.4.3+4:natives-windows-arm64",
                "rules": [{ "action": "allow", "os": { "name": "windows" } }],
                "downloads": { "artifact": {
                    "path": "org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4-natives-windows-arm64.jar",
                    "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4-natives-windows-arm64.jar",
                    "sha1": "2222222222222222222222222222222222222222",
                    "size": 10
                }}
            },
            {
                "name": "org.lwjgl:lwjgl:3.4.3+4:natives-windows",
                "rules": [{ "action": "allow", "os": { "name": "windows" } }],
                "downloads": { "artifact": {
                    "path": "org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4-natives-windows.jar",
                    "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4-natives-windows.jar",
                    "sha1": "3333333333333333333333333333333333333333",
                    "size": 10
                }}
            },
            {
                "name": "org.lwjgl:lwjgl-sdl:3.4.3+4:natives-windows-x86",
                "downloads": { "artifact": {
                    "path": "org/lwjgl/lwjgl-sdl/3.4.3+4/lwjgl-sdl-3.4.3+4-natives-windows-x86.jar",
                    "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-sdl/3.4.3+4/lwjgl-sdl-3.4.3+4-natives-windows-x86.jar",
                    "sha1": "4444444444444444444444444444444444444444",
                    "size": 10
                }}
            },
            {
                "name": "org.lwjgl:lwjgl-sdl:3.4.3+4:natives-windows-x86_64",
                "downloads": { "artifact": {
                    "path": "org/lwjgl/lwjgl-sdl/3.4.3+4/lwjgl-sdl-3.4.3+4-natives-windows-x86_64.jar",
                    "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-sdl/3.4.3+4/lwjgl-sdl-3.4.3+4-natives-windows-x86_64.jar",
                    "sha1": "5555555555555555555555555555555555555555",
                    "size": 10
                }}
            },
            {
                "name": "org.lwjgl:lwjgl:3.4.3+4:natives-linux",
                "downloads": { "artifact": {
                    "path": "org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4-natives-linux.jar",
                    "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4-natives-linux.jar",
                    "sha1": "6666666666666666666666666666666666666666",
                    "size": 10
                }}
            },
            {
                "name": "org.lwjgl:lwjgl:3.4.3+4:natives-macos-arm64",
                "downloads": { "artifact": {
                    "path": "org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4-natives-macos-arm64.jar",
                    "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4-natives-macos-arm64.jar",
                    "sha1": "7777777777777777777777777777777777777777",
                    "size": 10
                }}
            },
            {
                "name": "ca.weblite:java-objc-bridge:1.1",
                "rules": [{ "action": "allow", "os": { "name": "windows", "arch": "arm64" } }],
                "downloads": { "artifact": {
                    "path": "ca/weblite/java-objc-bridge/1.1/java-objc-bridge-1.1.jar",
                    "url": "https://libraries.minecraft.net/ca/weblite/java-objc-bridge/1.1/java-objc-bridge-1.1.jar",
                    "sha1": "8888888888888888888888888888888888888888",
                    "size": 10
                }}
            },
            {
                "name": "com.example:win-x64-only:1.0",
                "rules": [{ "action": "allow", "os": { "name": "windows", "arch": "x86_64" } }],
                "downloads": { "artifact": {
                    "path": "com/example/win-x64-only/1.0/win-x64-only-1.0.jar",
                    "url": "https://libraries.minecraft.net/com/example/win-x64-only/1.0/win-x64-only-1.0.jar",
                    "sha1": "9999999999999999999999999999999999999999",
                    "size": 10
                }}
            }
        ],
        "arguments": {
            "jvm": [
                "-Djava.library.path=${natives_directory}",
                "-cp",
                "${classpath}"
            ],
            "game": ["--version", "${version_name}"]
        }
    });

    let built = plan::windows(&version, None).expect("plan builds cleanly");
    let dests: Vec<&str> = built.files.iter().map(|f| f.destination.as_str()).collect();
    assert!(dests.contains(&"libraries/org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4.jar"));
    assert!(dests.contains(&"libraries/org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4-natives-windows.jar"));
    assert!(dests.contains(&"libraries/org/lwjgl/lwjgl-sdl/3.4.3+4/lwjgl-sdl-3.4.3+4-natives-windows-x86_64.jar"));
    assert!(dests.contains(&"libraries/com/example/win-x64-only/1.0/win-x64-only-1.0.jar"));
    assert!(!dests.iter().any(|d| d.contains("natives-windows-arm64")));
    assert!(!dests.iter().any(|d| d.ends_with("natives-windows-x86.jar")));
    assert!(!dests.iter().any(|d| d.contains("natives-linux")));
    assert!(!dests.iter().any(|d| d.contains("natives-macos")));
    assert!(!dests.iter().any(|d| d.contains("java-objc-bridge")));
}

#[test]
fn launch_classpath_guard_and_jvm_arg_order() {
    let fake_art = Artifact {
        url: "https://libraries.minecraft.net/dummy.jar".into(),
        sha1: "1111111111111111111111111111111111111111".into(),
        size: 10,
        path: None,
    };
    let built = plan::Plan {
        version: "26.3".into(),
        kind: "release".into(),
        main_class: "net.minecraft.client.main.Main".into(),
        java_major: 25,
        asset_index: fake_art.clone(),
        asset_index_id: "26".into(),
        arguments: json!({
            "jvm": [
                "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump",
                "-Djava.library.path=${natives_directory}",
                "-cp",
                "${classpath}"
            ],
            "game": ["--username", "${auth_player_name}"]
        }),
        legacy_arguments: None,
        logging_argument: None,
        files: vec![
            plan::File {
                destination: "libraries/org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4-natives-windows-arm64.jar".into(),
                artifact: fake_art.clone(),
            },
            plan::File {
                destination: "libraries/org/lwjgl/lwjgl/3.4.3+4/lwjgl-3.4.3+4-natives-windows.jar".into(),
                artifact: fake_art.clone(),
            },
            plan::File {
                destination: "libraries/org/lwjgl/lwjgl-sdl/3.4.3+4/lwjgl-sdl-3.4.3+4-natives-windows-x86_64.jar".into(),
                artifact: fake_art.clone(),
            },
            plan::File {
                destination: "libraries/org/lwjgl/lwjgl-sdl/3.4.3+4/lwjgl-sdl-3.4.3+4-natives-windows-x86.jar".into(),
                artifact: fake_art,
            },
        ],
        natives: vec![],
    };
    let session = Session {
        profile: Profile {
            id: "12345678-1234-5678-1234-567812345678".into(),
            name: "SolaraPlayer".into(),
            active: true,
            owned: true,
        },
        access_token: "secret-token".into(),
        xuid: None,
    };
    let args = launch::build_args_with_options(
        &built,
        &PathBuf::from("/tmp/solara-runtime"),
        &PathBuf::from("/tmp/solara-game"),
        &session,
        4096,
        Some("10.0"),
        Some("-XX:+UseZGC"),
        None,
        false,
    )
    .expect("args build cleanly");
    let strings: Vec<String> = args
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    let cp_idx = strings.iter().position(|s| s == "-cp").expect("-cp exists");
    let cp_val = &strings[cp_idx + 1];
    assert!(cp_val.contains("natives-windows.jar"));
    assert!(cp_val.contains("natives-windows-x86_64.jar"));
    assert!(!cp_val.contains("natives-windows-arm64"));
    assert!(!cp_val.contains("natives-windows-x86.jar"));

    let xmx_idx = strings.iter().position(|s| s == "-Xmx4096M").expect("-Xmx exists");
    let extra_idx = strings.iter().position(|s| s == "-XX:+UseZGC").expect("extra arg exists");
    let main_idx = strings.iter().position(|s| s == "net.minecraft.client.main.Main").expect("main class exists");
    assert!(cp_idx < xmx_idx, "arguments.jvm must appear before -Xmx");
    assert!(xmx_idx < extra_idx, "-Xmx must appear before extra_jvm_args");
    assert!(extra_idx < main_idx, "extra_jvm_args must appear before main_class");
}

#[test]
fn game_command_skips_create_no_window_for_javaw_and_clears_java_options() {
    assert!(!launch::should_apply_create_no_window("C:\\Java\\bin\\javaw.exe"));
    assert!(!launch::should_apply_create_no_window("/opt/java/bin/javaw"));
    assert!(launch::should_apply_create_no_window("C:\\Java\\bin\\java.exe"));
    assert!(launch::should_apply_create_no_window("java"));

    let cmd = launch::build_game_command(
        "C:\\Java\\bin\\javaw.exe",
        &[],
        &PathBuf::from("."),
    );
    let has_removed_java_opts = cmd
        .as_std()
        .get_envs()
        .any(|(k, v)| k == "_JAVA_OPTIONS" && v.is_none());
    assert!(has_removed_java_opts, "_JAVA_OPTIONS must be explicitly removed");
}

#[tokio::test]
async fn console_java_probe_resolves_sibling_and_cached_requires_javaw() {
    let dir = std::env::temp_dir().join(format!("solara-v41-java-{}", uuid::Uuid::new_v4()));
    let bin = dir.join("java-25/bin");
    let server = bin.join("server");
    tokio::fs::create_dir_all(&server).await.unwrap();

    let javaw = bin.join("javaw.exe");
    let java = bin.join("java.exe");
    let jvm = server.join("jvm.dll");
    let java_bytes = b"#!/bin/sh\necho 'openjdk version \"25.0.5\"' >&2\n";
    let jvm_bytes = b"mock-jvm-dll";
    tokio::fs::write(&javaw, b"gui-stub").await.unwrap();
    tokio::fs::write(&java, java_bytes).await.unwrap();
    tokio::fs::write(&jvm, jvm_bytes).await.unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(&java, std::fs::Permissions::from_mode(0o755))
            .await
            .unwrap();
        assert_eq!(launch::console_java_probe_path(&javaw), java);
        launch::java_check(javaw.to_str().unwrap(), 25)
            .await
            .expect("sibling java.exe satisfies java_check");
        assert!(launch::java_check(javaw.to_str().unwrap(), 26).await.is_err());
    }

    let runtime_json = json!({
        "major": 25,
        "version": "25.0.5",
        "archiveSha256": sha256_hex(b"zip"),
        "javaSha256": sha256_hex(java_bytes),
        "jvmSha256": sha256_hex(jvm_bytes),
    });
    tokio::fs::write(
        dir.join("java-25/solara-runtime.json"),
        serde_json::to_vec(&runtime_json).unwrap(),
    )
    .await
    .unwrap();

    let runtimes = java::Runtimes::new(dir.clone());
    assert_eq!(runtimes.cached(25).await.unwrap(), Some(javaw.clone()));

    tokio::fs::remove_file(&javaw).await.unwrap();
    assert_eq!(runtimes.cached(25).await.unwrap(), None);
    #[cfg(unix)]
    assert!(launch::java_check(javaw.to_str().unwrap(), 25).await.is_err());

    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[test]
fn azul_zulu_origin_and_package_validation() {
    let valid = reqwest::Url::parse(
        "https://cdn.azul.com/zulu/bin/zulu25.38.109-ca-jre25.0.5.0.3-win_x64.zip",
    )
    .unwrap();
    assert!(java::zulu_package_url(&valid));
    assert!(java::package_url(&valid, 25));
    assert!(java::redirect_url(&valid, 25));

    for bad in [
        "http://cdn.azul.com/zulu/bin/zulu25.38.109-ca-jre25.0.5.0.3-win_x64.zip",
        "https://cdn.azul.com/zulu/bin/zulu8.98.0.111-ca-jre8.0.512-win_i686.zip",
        "https://cdn.azul.com/other/zulu25-win_x64.zip",
        "https://cdn.azul.com/zulu/bin/../secret/zulu25-win_x64.zip",
        "https://evil.example/zulu/bin/zulu25-win_x64.zip",
    ] {
        let u = reqwest::Url::parse(bad).unwrap();
        assert!(!java::zulu_package_url(&u), "should reject {bad}");
    }

    let detail = json!({
        "name": "zulu25.38.109-ca-jre25.0.5.0.3-win_x64.zip",
        "java_version": [25, 0, 5],
        "download_url": "https://cdn.azul.com/zulu/bin/zulu25.38.109-ca-jre25.0.5.0.3-win_x64.zip",
        "sha256_hash": "34ce17a0319cca98f76d1d4e7bc9e5bb13c5d76744a70cdbb82a2edcfd31f8d1",
        "size": 58634400
    });
    let pkg = java::select_zulu_package(&detail, 25).expect("valid Zulu package parses");
    assert_eq!(pkg.version, "25.0.5");
    assert_eq!(pkg.approx_size, 58634400);
    assert_eq!(pkg.exact_size, None);
    assert!(java::select_zulu_package(&detail, 21).is_err(), "rejects wrong major");

    let mut bad_hash = detail.clone();
    bad_hash["sha256_hash"] = json!("not-a-sha256");
    assert!(java::select_zulu_package(&bad_hash, 25).is_err());

    let mut huge = detail;
    huge["size"] = json!(900_000_000u64);
    assert!(java::select_zulu_package(&huge, 25).is_err());
}

#[tokio::test]
async fn resolve_java_auto_falls_back_when_custom_java_missing_or_wrong_major() {
    let managed = PathBuf::from("C:\\Solara\\java-25\\bin\\javaw.exe");

    let custom_ok = crate::engine::resolve_java_path_with(
        "C:\\Custom\\Java25\\bin\\javaw.exe",
        25,
        |_, _| async { Ok(()) },
        async { Ok::<PathBuf, anyhow::Error>(PathBuf::from("unused")) },
    )
    .await
    .unwrap();
    assert_eq!(custom_ok, "C:\\Custom\\Java25\\bin\\javaw.exe");

    let missing_fallback = crate::engine::resolve_java_path_with(
        "C:\\Missing\\Java\\bin\\javaw.exe",
        25,
        |_, _| async { anyhow::bail!("missing binary") },
        {
            let managed = managed.clone();
            async move { Ok::<PathBuf, anyhow::Error>(managed) }
        },
    )
    .await
    .unwrap();
    assert_eq!(missing_fallback, managed.to_string_lossy());

    let wrong_major_fallback = crate::engine::resolve_java_path_with(
        "C:\\Old\\Java8\\bin\\javaw.exe",
        25,
        |_, _| async { anyhow::bail!("Java 25 required, found Java 8") },
        {
            let managed = managed.clone();
            async move { Ok::<PathBuf, anyhow::Error>(managed) }
        },
    )
    .await
    .unwrap();
    assert_eq!(wrong_major_fallback, managed.to_string_lossy());
}

#[tokio::test]
async fn crash_diagnosis_parses_hs_err_atio6axx_nvoglv_and_clean_exit() {
    let dir = std::env::temp_dir().join(format!("solara-v41-crash-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir_all(dir.join("logs")).await.unwrap();

    assert!(launch::inspect_crash_diagnosis(&dir, Some(0), "").is_none());

    tokio::fs::write(
        dir.join("hs_err_pid8120.log"),
        "# A fatal error has been detected by the Java Runtime Environment:\n#\n#  EXCEPTION_ACCESS_VIOLATION (0xc0000005) at pc=0x00007ff912345678, pid=8120, tid=8124\n#\n# JRE version: OpenJDK Runtime Environment Temurin-25.0.5+8\n# Problematic frame:\n# C  [atio6axx.dll+0x193ca0]\n",
    )
    .await
    .unwrap();

    let amd = launch::inspect_crash_diagnosis(&dir, Some(-1073741819), "")
        .expect("detects AMD atio6axx.dll crash");
    assert_eq!(amd.code, "amd-opengl-driver");
    assert_eq!(amd.hs_err_file.as_deref(), Some("hs_err_pid8120.log"));
    assert!(amd.summary.contains("atio6axx.dll"));
    assert!(amd.advice.iter().any(|a| a.contains("High Performance")));

    tokio::fs::remove_file(dir.join("hs_err_pid8120.log"))
        .await
        .unwrap();
    tokio::fs::write(
        dir.join("logs/latest.log"),
        "FATAL: Exception in nvoglv64.dll during OpenGL context creation",
    )
    .await
    .unwrap();
    let nv = launch::inspect_crash_diagnosis(&dir, Some(1), "")
        .expect("detects nvoglv64.dll from latest.log fallback");
    assert_eq!(nv.code, "nvidia-opengl-driver");

    let jv = launch::inspect_crash_diagnosis(
        &dir,
        Some(1),
        "java.lang.UnsupportedClassVersionError: compiled by a more recent version of the Java Runtime",
    )
    .expect("detects Java mismatch");
    assert_eq!(jv.code, "java-mismatch");

    let _ = tokio::fs::remove_dir_all(&dir).await;
}

fn build_fixture_java_zip() -> Vec<u8> {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut cursor);
        let opts = zip::write::SimpleFileOptions::default();
        for (name, content) in [
            ("zulu25-win_x64/bin/java.exe", b"MZ-java-exe".as_slice()),
            ("zulu25-win_x64/bin/javaw.exe", b"MZ-javaw-exe".as_slice()),
            ("zulu25-win_x64/bin/server/jvm.dll", b"MZ-jvm-dll".as_slice()),
            (
                "zulu25-win_x64/release",
                b"IMPLEMENTOR=\"Azul Systems, Inc.\"\nJAVA_VERSION=\"25.0.5\"\n",
            ),
        ] {
            zip.start_file(name, opts).unwrap();
            zip.write_all(content).unwrap();
        }
        zip.finish().unwrap();
    }
    cursor.into_inner()
}

async fn spawn_fixture_http_server(
    routes: Vec<(&'static str, Vec<u8>)>,
) -> (reqwest::Url, tokio::task::JoinHandle<()>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let routes = std::sync::Arc::new(routes);
    let handle = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let routes = routes.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 2048];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]);
                let path = req
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("/");
                if let Some((_, body)) = routes.iter().find(|(p, _)| *p == path) {
                    let header = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/zip\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(header.as_bytes()).await;
                    let _ = stream.write_all(body).await;
                } else {
                    let _ = stream
                        .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                        .await;
                }
            });
        }
    });
    let base = reqwest::Url::parse(&format!("http://{addr}")).unwrap();
    (base, handle)
}

#[tokio::test]
async fn zulu_rounded_metadata_size_downloads_and_extracts_verified_runtime() {
    let dir = std::env::temp_dir().join(format!("solara-v41-zulu-dl-{}", uuid::Uuid::new_v4()));
    let zip_bytes = build_fixture_java_zip();
    let zip_sha = sha256_hex(&zip_bytes);
    // Simulate Azul's 100-byte rounded metadata size (strictly different from zip_bytes.len()).
    let rounded_size = ((zip_bytes.len() as u64 + 99) / 100) * 100 + 100;
    assert_ne!(rounded_size, zip_bytes.len() as u64);

    let (base, server) = spawn_fixture_http_server(vec![("/zulu25.zip", zip_bytes)]).await;
    let candidate = java::Package {
        version: "25.0.5".into(),
        url: base.join("/zulu25.zip").unwrap(),
        approx_size: rounded_size,
        exact_size: None,
        sha256: zip_sha,
    };

    let runtimes = java::Runtimes::new(dir.clone());
    let cancel = crate::game_core::download::Cancellation::default();
    let mut phases = Vec::new();
    let installed = runtimes
        .ensure_from_candidates_test(25, vec![candidate], &cancel, |phase, _, _| {
            phases.push(phase.to_string());
        })
        .await
        .expect("Zulu candidate with rounded approx_size must download and verify via SHA-256");

    assert!(installed.ends_with("java-25/bin/javaw.exe") || installed.ends_with("java-25\\bin\\javaw.exe"));
    assert_eq!(runtimes.cached(25).await.unwrap(), Some(installed.clone()));
    assert!(phases.iter().any(|p| p == "Downloading Java"));
    assert!(phases.iter().any(|p| p == "Java is ready"));

    // Empty/corrupted javaw.exe must invalidate cached() even if java.exe and jvm.dll hashes match.
    tokio::fs::write(&installed, b"").await.unwrap();
    assert_eq!(runtimes.cached(25).await.unwrap(), None);

    server.abort();
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn corrupted_or_mismatched_exact_size_candidate_falls_back_cleanly() {
    let dir = std::env::temp_dir().join(format!("solara-v41-java-fb-{}", uuid::Uuid::new_v4()));
    let good_zip = build_fixture_java_zip();
    let good_sha = sha256_hex(&good_zip);
    let good_len = good_zip.len() as u64;

    let (base, server) = spawn_fixture_http_server(vec![
        ("/corrupt.zip", b"not-a-valid-zip-stream".to_vec()),
        ("/good.zip", good_zip),
    ])
    .await;

    let bad_first = java::Package {
        version: "25.0.5".into(),
        url: base.join("/corrupt.zip").unwrap(),
        approx_size: 200,
        exact_size: None,
        sha256: "a".repeat(64),
    };
    let mismatched_exact = java::Package {
        version: "25.0.5".into(),
        url: base.join("/good.zip").unwrap(),
        approx_size: good_len + 10,
        exact_size: Some(good_len + 10),
        sha256: good_sha.clone(),
    };
    let good_second = java::Package {
        version: "25.0.5".into(),
        url: base.join("/good.zip").unwrap(),
        approx_size: good_len,
        exact_size: Some(good_len),
        sha256: good_sha,
    };

    let runtimes = java::Runtimes::new(dir.clone());
    let cancel = crate::game_core::download::Cancellation::default();
    let installed = runtimes
        .ensure_from_candidates_test(
            25,
            vec![bad_first, mismatched_exact, good_second],
            &cancel,
            |_, _, _| {},
        )
        .await
        .expect("falls back past bad checksum and wrong exact_size to verified candidate");
    assert!(tokio::fs::try_exists(&installed).await.unwrap());

    server.abort();
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn crash_diagnosis_checks_latest_log_with_stdout_and_filters_stale_files() {
    let dir = std::env::temp_dir().join(format!("solara-v41-stale-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir_all(dir.join("logs")).await.unwrap();

    tokio::fs::write(
        dir.join("logs/latest.log"),
        "[Render thread/FATAL]: Exception in nvoglv64.dll during OpenGL init",
    )
    .await
    .unwrap();

    // Non-empty stdout without error lines must still consult logs/latest.log.
    let diagnosed = launch::inspect_crash_diagnosis(
        &dir,
        Some(-1073741819),
        "[main/INFO]: Setting user: Steve\n[main/INFO]: LWJGL Version: 3.4.3\n",
    )
    .expect("detects nvoglv64.dll from latest.log even when stdout is non-empty");
    assert_eq!(diagnosed.code, "nvidia-opengl-driver");

    // A launch cutoff strictly in the future ignores older hs_err/latest.log files.
    tokio::fs::write(
        dir.join("hs_err_pid1111.log"),
        "# C  [atio6axx.dll+0x193ca0]\n",
    )
    .await
    .unwrap();
    let future_cutoff = std::time::SystemTime::now() + std::time::Duration::from_secs(60);
    assert!(
        launch::inspect_crash_diagnosis_since(&dir, Some(1), "[main/INFO]: exited cleanly?", Some(future_cutoff))
            .is_none()
    );

    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[test]
fn plan_windows_filters_aarch_64_netty_classifiers() {
    let version = json!({
        "id": "26.3",
        "type": "release",
        "mainClass": "net.minecraft.client.main.Main",
        "javaVersion": { "majorVersion": 25 },
        "downloads": {
            "client": {
                "url": "https://piston-data.mojang.com/v1/objects/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/client.jar",
                "sha1": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "size": 100
            }
        },
        "assetIndex": {
            "id": "26",
            "url": "https://piston-meta.mojang.com/v1/packages/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb/26.json",
            "sha1": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "size": 50
        },
        "libraries": [
            {
                "name": "io.netty:netty-transport-native-epoll:4.2.7.Final:linux-aarch_64",
                "downloads": {
                    "artifact": {
                        "path": "io/netty/netty-transport-native-epoll/4.2.7.Final/netty-transport-native-epoll-4.2.7.Final-linux-aarch_64.jar",
                        "url": "https://libraries.minecraft.net/io/netty/netty-transport-native-epoll/4.2.7.Final/netty-transport-native-epoll-4.2.7.Final-linux-aarch_64.jar",
                        "sha1": "1111111111111111111111111111111111111111",
                        "size": 10
                    }
                }
            },
            {
                "name": "io.netty:netty-codec-http:4.2.7.Final",
                "downloads": {
                    "artifact": {
                        "path": "io/netty/netty-codec-http/4.2.7.Final/netty-codec-http-4.2.7.Final.jar",
                        "url": "https://libraries.minecraft.net/io/netty/netty-codec-http/4.2.7.Final/netty-codec-http-4.2.7.Final.jar",
                        "sha1": "2222222222222222222222222222222222222222",
                        "size": 10
                    }
                }
            }
        ]
    });

    let p = plan::windows(&version, None).unwrap();
    let names: Vec<_> = p.files.iter().map(|l| l.destination.as_str()).collect();
    assert!(!names.iter().any(|n| n.contains("aarch_64")));
    assert!(names.iter().any(|n| n.contains("netty-codec-http")));
}
