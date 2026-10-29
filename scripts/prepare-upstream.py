#!/usr/bin/env python3
"""Fetch pinned Theseus sources locally and apply documented Solara changes.
Fetched code is ignored. Licence review is required before redistribution.
"""
import io, json, os, re, runpy, shutil, subprocess, tarfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
REV='e7757b14ed04e824ac7e8b8b05652d4290df721a'
DEST=ROOT/'.upstream/modrinth'
CACHE=Path(os.environ.get('SOLARA_UPSTREAM_CACHE','/tmp/solara-modrinth-inspect'))
if not (CACHE/'.git').exists():
    CACHE.parent.mkdir(parents=True,exist_ok=True)
    subprocess.run(['git','clone','--bare','--filter=blob:none','https://github.com/modrinth/code.git',str(CACHE)],check=True)
subprocess.run(['git','-C',str(CACHE),'fetch','--depth','1','origin',REV],check=True)
paths=['Cargo.toml','Cargo.lock','COPYING.md','.gitattributes','packages/app-lib','packages/ariadne','packages/daedalus','packages/async-minecraft-ping','packages/modrinth-content-management','packages/path-util','packages/serde-binhum','apps/app/LICENSE']
raw=subprocess.check_output(['git','-C',str(CACHE),'archive',REV,*paths])
if DEST.exists(): shutil.rmtree(DEST)
DEST.mkdir(parents=True)
with tarfile.open(fileobj=io.BytesIO(raw)) as archive: archive.extractall(DEST,filter='data')
workspace=DEST/'Cargo.toml';s=workspace.read_text();s=re.sub(r'members = \[.*?\]', 'members = ["packages/app-lib", "packages/ariadne", "packages/daedalus", "packages/async-minecraft-ping", "packages/modrinth-content-management", "packages/path-util", "packages/serde-binhum"]',s,count=1,flags=re.S);workspace.write_text(s)
lib=DEST/'packages/app-lib'
shutil.copyfile(lib/'.env.prod',lib/'.env')
manifest=lib/'Cargo.toml';s=manifest.read_text();s+='\n[target.\'cfg(windows)\'.dependencies.keyring]\nversion = "3.6.3"\nfeatures = ["windows-native"]\n';manifest.write_text(s)
main=lib/'src/lib.rs';s=main.read_text().replace('mod logger;','mod logger;\nmod solara_vault;').replace('"modrinth/theseus/"','"solara/theseus/"').replace('support@modrinth.com','https://www.solara-inc.in/contact');main.write_text(s)
(lib/'src/solara_vault.rs').write_text('''
#[cfg(windows)]
pub fn put(name: &str, value: &str) -> crate::Result<String> {
    let entry = keyring::Entry::new("in.solara.launcher.minecraft", name).map_err(|_| anyhow::anyhow!("Windows credential vault unavailable"))?;
    entry.set_password(value).map_err(|_| anyhow::anyhow!("Windows credential vault write failed"))?;
    Ok(format!("solara-vault:{name}"))
}
#[cfg(windows)]
pub fn get(reference: &str) -> crate::Result<String> {
    let name = reference.strip_prefix("solara-vault:").ok_or_else(|| anyhow::anyhow!("Unprotected legacy credential rejected; sign in again"))?;
    keyring::Entry::new("in.solara.launcher.minecraft", name).and_then(|e| e.get_password()).map_err(|_| anyhow::anyhow!("Windows credential vault read failed; sign in again").into())
}
#[cfg(not(windows))]
pub fn put(_: &str, _: &str) -> crate::Result<String> { Err(anyhow::anyhow!("Minecraft credential storage currently requires Windows Credential Manager").into()) }
#[cfg(not(windows))]
pub fn get(_: &str) -> crate::Result<String> { Err(anyhow::anyhow!("Minecraft credential storage currently requires Windows Credential Manager").into()) }
''')
auth=lib/'src/state/minecraft_auth.rs';s=auth.read_text();s=re.sub(r'#\[tracing::instrument(?:\([^\n]*\))?\]\n','',s)
s=s.replace('access_token: x.access_token,','access_token: crate::solara_vault::get(&x.access_token)?,').replace('refresh_token: x.refresh_token,','refresh_token: crate::solara_vault::get(&x.refresh_token)?,')
s=s.replace('let profile = self.maybe_online_profile().await;\n        let expires', 'let protected_access = crate::solara_vault::put(&format!("{}:access", self.offline_profile.id), &self.access_token)?;\n        let protected_refresh = crate::solara_vault::put(&format!("{}:refresh", self.offline_profile.id), &self.refresh_token)?;\n        let profile = self.maybe_online_profile().await;\n        let expires')
s=s.replace('            self.access_token,\n            self.refresh_token,','            protected_access,\n            protected_refresh,')
s=s.replace('SigningKey::from_pkcs8_pem(&x.private_key)','SigningKey::from_pkcs8_pem(&crate::solara_vault::get(&x.private_key)?)').replace('token: x.token,','token: crate::solara_vault::get(&x.token)?,')
s=s.replace('let display_claims = serde_json::to_string(&self.token.display_claims)?;','let key = crate::solara_vault::put("device:private", &key)?;\n        let protected_device_token = crate::solara_vault::put("device:token", &self.token.token)?;\n        let display_claims = serde_json::to_string(&self.token.display_claims)?;')
s=s.replace('            self.token.token,\n            display_claims,','            protected_device_token,\n            display_claims,')
s=s.replace('const MICROSOFT_CLIENT_ID: &str = "00000000402b5328";','const MICROSOFT_CLIENT_ID: &str = match option_env!("SOLARA_MICROSOFT_CLIENT_ID") { Some(id) => id, None => "" };')
s=s.replace('Modrinth App (support@modrinth.com; https://modrinth.com/app)','Solara Launcher (https://launcher.solara-inc.in)')
# try_fold constructs credentials synchronously; make fallible vault reads inside its async block.
s=s.replace('let mut credentials = Self {\n                offline_profile:', 'let access_token = crate::solara_vault::get(&x.access_token);\n            let refresh_token = crate::solara_vault::get(&x.refresh_token);\n            async move {\n            let mut credentials = Self {\n                offline_profile:',1)
# This replacement is scoped to the get_all closure only in this pinned source.
start=s.index('    pub async fn get_all(');end=s.index('    pub async fn upsert(',start);chunk=s[start:end]
chunk=chunk.replace('access_token: crate::solara_vault::get(&x.access_token)?,','access_token: access_token?,').replace('refresh_token: crate::solara_vault::get(&x.refresh_token)?,','refresh_token: refresh_token?,').replace('            async move {\n                credentials.refresh(exec).await.ok();','                credentials.refresh(exec).await.ok();')
chunk=chunk.replace('access_token: access_token?,','access_token: access_token.map_err(|_| sqlx::Error::Protocol("Windows credential vault read failed".into()))?,').replace('refresh_token: refresh_token?,','refresh_token: refresh_token.map_err(|_| sqlx::Error::Protocol("Windows credential vault read failed".into()))?,')
s=s[:start]+chunk+s[end:];auth.write_text(s)
p=lib/'src/api/minecraft_auth.rs';s=p.read_text();s=re.sub(r'#\[tracing::instrument(?:\([^\n]*\))?\]\n','',s);p.write_text(s)
# Never expose upstream app identity in Discord; the Solara wrapper owns optional presence.
p=lib/'src/state/discord.rs';s=p.read_text().replace('"1123683254248148992"','"0"');s=s.replace('        // Attempt to connect if not connected.','        return Ok(());\n        #[allow(unreachable_code)]\n        // Solara owns Discord IPC separately.\n        // Attempt to connect if not connected.');p.write_text(s)
# Disable third-party friends sockets on startup. Keep game installation/metadata/provider functionality.
p=lib/'src/state/mod.rs';s=p.read_text();a=s.index('            let _ = state\n                .presence\n                .friends_socket');b=s.index('            let _ = FriendsSocket::socket_loop(Arc::clone(state)).await;',a)+len('            let _ = FriendsSocket::socket_loop(Arc::clone(state)).await;');s=s[:a]+s[b:];p.write_text(s)
# Authentication response bodies may contain credentials: do not propagate them through diagnostics.
p=lib/'src/state/minecraft_auth.rs';s=p.read_text().replace('Status Code: {status_code} Body: {raw}','Status Code: {status_code}');p.write_text(s)
(DEST/'SOLARA_PATCHES.json').write_text(json.dumps({'upstream':REV,'changes':['Solara user-agent','Windows credential vault references instead of SQLite secrets','Reject legacy plaintext tokens','No auth argument tracing/response bodies','Explicit Microsoft client ID','Disable upstream Discord and friends startup'],'distribution':'blocked pending transitive package licence review'},indent=2))
# Allow a locally downloaded, checksum-verified Gradle distribution without changing engine behavior.
p=lib/'build.rs';s=p.read_text().replace('let gradle_path = fs::canonicalize(', 'let gradle_path = std::env::var_os("SOLARA_GRADLE_BIN").map(PathBuf::from).unwrap_or_else(|| fs::canonicalize(').replace('    .unwrap();\n\n    let mut build_dir_str', '    .unwrap());\n\n    let mut build_dir_str');p.write_text(s)
runpy.run_path(str(ROOT/'scripts/patch-content-events.py'))['patch'](lib)
# Archive extraction preserves upstream LF bytes. Spotless cannot reliably infer
# Git attributes for this ignored, extracted tree on Windows; use upstream's LF
# policy explicitly while retaining every formatting, compilation and test task.
p=lib/'java/build.gradle.kts';s=p.read_text(encoding='utf-8')
if s.count('spotless {\n') != 1: raise RuntimeError('Pinned Java Spotless block changed')
s=s.replace('spotless {\n','spotless {\n    lineEndings = com.diffplug.spotless.LineEnding.UNIX\n',1)
p.write_text(s,encoding='utf-8',newline='\n')
p=DEST/'SOLARA_PATCHES.json';patches=json.loads(p.read_text());patches['changes'].append('Preserve upstream LF policy for archived Java sources on Windows; retain Spotless checks');p.write_text(json.dumps(patches,indent=2)+'\n')
print('Prepared pinned engine and Solara patches:',DEST)
