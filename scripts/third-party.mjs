// Build notices from the locked Windows dependency graph; never invent grants.
import { execFileSync } from 'node:child_process';
import { readFile, writeFile, mkdir, readdir, copyFile, access } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';

const root = process.cwd();
const output = path.resolve(process.argv[2] || 'dist/third-party');
const readJson = async (file) => JSON.parse(await readFile(file, 'utf8'));
const exists = async (p) => {
  try {
    await access(p);
    return true;
  } catch {
    return false;
  }
};

const supplements = await readJson('third-party/licence-supplements/sources.json');
const migration = (await exists('docs/engine-migration.json'))
  ? await readJson('docs/engine-migration.json')
  : { inheritedEngineRemoved: true };

const metadata = JSON.parse(
  execFileSync(
    'cargo',
    ['metadata', '--locked', '--format-version', '1', '--filter-platform', 'x86_64-pc-windows-msvc'],
    { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 },
  ),
);

const ids = new Set(metadata.resolve.nodes.map((n) => n.id));
const retired = new Set([
  'theseus',
  'ariadne',
  'modrinth-content-management',
  'path-util',
  'serde-binhum',
  'daedalus',
  'async-minecraft-ping',
]);
if (migration.inheritedEngineRemoved === true && metadata.packages.some((p) => retired.has(p.name))) {
  throw Error('Retired engine dependency remains in the lockfile');
}

const blockedNpm = new Set([
  '@intercom/messenger-js-sdk',
  '@stripe/stripe-js',
  'stripe',
  '@sentry/vue',
  'apexcharts',
  'vue3-apexcharts',
]);

async function texts(dir, depth = 0) {
  const found = [];
  if (!(await exists(dir))) return found;
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    if (entry.isSymbolicLink()) continue;
    const file = path.join(dir, entry.name);
    if (entry.isDirectory() && depth < 2 && entry.name !== 'node_modules' && entry.name !== 'target') {
      found.push(...(await texts(file, depth + 1)));
    }
    if (entry.isFile() && /^(licen[sc]e|copying|notice|copyright|unlicense)($|[.\-_])/i.test(entry.name)) {
      found.push(file);
    }
  }
  return found;
}

const report = [];

async function record(ecosystem, name, version, licence, source, dir, options = {}) {
  if (ecosystem === 'npm' && blockedNpm.has(name)) {
    throw Error(`Disallowed commercial/telemetry dependency detected: ${name}@${version}`);
  }
  const hasRealLicenseFile = Boolean(options.licenseFile && (await exists(path.join(dir, options.licenseFile))));
  let effectiveLicence = licence;
  if (!effectiveLicence && hasRealLicenseFile) {
    effectiveLicence = `SEE LICENSE IN ${options.licenseFile}`;
  }
  if (!effectiveLicence && options.workspaceFallbackLicence) {
    effectiveLicence = options.workspaceFallbackLicence;
  }
  if (!effectiveLicence || (/UNLICENSED|SEE LICENSE/i.test(effectiveLicence) && !hasRealLicenseFile)) {
    throw Error(`Review required: ${name}@${version}`);
  }

  const dest = path.join(output, ecosystem, `${name.replaceAll('/', '_')}-${version}`);
  await mkdir(dest, { recursive: true });
  const files = await texts(dir);
  let supplement = options.supplement || null;

  // Published platform-tool archives share their parent package's licence.
  // Only use an installed parent of the exact same locked version.
  const parent = name.startsWith('@esbuild/')
    ? 'esbuild'
    : name.startsWith('@rollup/rollup-')
      ? 'rollup'
      : name.startsWith('@tauri-apps/cli-')
        ? '@tauri-apps/cli'
        : null;
  if (!files.length && ecosystem === 'npm' && parent) {
    const parentCandidates = [path.join(root, 'node_modules', parent)];
    if (await exists(path.join(root, 'node_modules/.pnpm'))) {
      const encoded = `${parent.replace('/', '+')}@${version}`;
      parentCandidates.unshift(path.join(root, 'node_modules/.pnpm', encoded, 'node_modules', parent));
    }
    for (const parentDir of parentCandidates) {
      if (!(await exists(path.join(parentDir, 'package.json')))) continue;
      const info = await readJson(path.join(parentDir, 'package.json'));
      if (info.version !== version) throw Error('Platform tool and parent version mismatch');
      files.push(...(await texts(parentDir)));
      supplement = { parentPackage: `${parent}@${version}` };
      break;
    }
  }

  if (!files.length && options.workspaceRootFallback) {
    files.push(path.join(root, 'LICENSE'));
    if (await exists(path.join(root, 'THIRD_PARTY_NOTICES.md'))) {
      files.push(path.join(root, 'THIRD_PARTY_NOTICES.md'));
    }
    supplement = supplement || { note: options.workspaceFallbackNote || 'Upstream GPL-3.0-only workspace package' };
  }

  if (!files.length) {
    supplement = supplements[`${name}@${version}`];
    if (!supplement) throw Error(`Missing licence text: ${name}@${version}`);
    const local = path.join(root, 'third-party/licence-supplements', supplement.file);
    if (createHash('sha256').update(await readFile(local)).digest('hex') !== supplement.sha256) {
      throw Error('Changed licence supplement');
    }
    files.push(local);
  }

  const notices = [];
  for (const [index, file] of files.entries()) {
    const filename = `${index}-${path.basename(file)}`;
    await copyFile(file, path.join(dest, filename));
    notices.push(filename);
  }
  report.push({ ecosystem, name, version, licence: effectiveLicence, source, notices, supplement });
}

for (const p of metadata.packages.filter((p) => ids.has(p.id))) {
  const pkgDir = path.dirname(p.manifest_path);
  if (p.source) {
    await record(
      'cargo',
      p.name,
      p.version,
      p.license,
      `https://crates.io/api/v1/crates/${p.name}/${p.version}/download`,
      pkgDir,
      { licenseFile: p.license_file },
    );
  } else if (p.name !== 'solara-launcher') {
    const relDir = path.relative(root, pkgDir).replaceAll('\\', '/');
    await record(
      'cargo-workspace',
      p.name,
      p.version,
      p.license,
      p.repository || 'https://github.com/modrinth/code/tree/1b27f67b4/' + relDir,
      pkgDir,
      {
        licenseFile: p.license_file,
        workspaceRootFallback: true,
        workspaceFallbackLicence: p.license || 'GPL-3.0-only (upstream workspace crate; Cargo.toml omits license)',
        workspaceFallbackNote: `Local workspace crate at ${relDir}; see THIRD_PARTY_NOTICES.md`,
      },
    );
  }
}

function parsePnpmPackagesSection(yamlText) {
  const entries = [];
  const lines = yamlText.split(/\r?\n/);
  let inPackages = false;
  let current = null;
  for (const line of lines) {
    if (!inPackages) {
      if (line === 'packages:') inPackages = true;
      continue;
    }
    if (line && !line.startsWith(' ')) {
      if (current) entries.push(current);
      break;
    }
    const header = line.match(/^  ('?)([^']+?)\1:\s*$/);
    if (header) {
      if (current) entries.push(current);
      current = { key: header[2], platformRestricted: false };
      continue;
    }
    if (current && (/^\s{4}(?:os|cpu|libc):/.test(line))) {
      current.platformRestricted = true;
    }
  }
  if (inPackages && current && !entries.includes(current)) entries.push(current);
  return entries;
}

if (await exists('pnpm-lock.yaml')) {
  const yamlText = await readFile('pnpm-lock.yaml', 'utf8');
  const pnpmEntries = parsePnpmPackagesSection(yamlText);
  const seenPnpm = new Set();
  for (const entry of pnpmEntries) {
    const atIdx = entry.key.lastIndexOf('@');
    if (atIdx <= 0) continue;
    const pkgName = entry.key.slice(0, atIdx);
    const pkgVersion = entry.key.slice(atIdx + 1).split('(')[0];
    if (blockedNpm.has(pkgName)) {
      throw Error(`Disallowed commercial/telemetry dependency detected in pnpm-lock.yaml: ${pkgName}@${pkgVersion}`);
    }
    const id = `${pkgName}@${pkgVersion}`;
    if (seenPnpm.has(id)) continue;
    seenPnpm.add(id);

    const storePrefix = `${pkgName.replace('/', '+')}@${pkgVersion}`;
    let pkgDir = path.join(root, 'node_modules/.pnpm', storePrefix, 'node_modules', pkgName);
    if (!(await exists(path.join(pkgDir, 'package.json')))) {
      const pnpmDir = path.join(root, 'node_modules/.pnpm');
      if (await exists(pnpmDir)) {
        const dirs = await readdir(pnpmDir);
        const match = dirs.find((d) => d === storePrefix || d.startsWith(`${storePrefix}_`));
        if (match) {
          pkgDir = path.join(pnpmDir, match, 'node_modules', pkgName);
        }
      }
    }
    if (!(await exists(path.join(pkgDir, 'package.json')))) {
      const flatDir = path.join(root, 'node_modules', pkgName);
      if (await exists(path.join(flatDir, 'package.json'))) {
        pkgDir = flatDir;
      } else if (entry.platformRestricted) {
        continue;
      } else {
        continue;
      }
    }
    const info = await readJson(path.join(pkgDir, 'package.json'));
    const licence = typeof info.license === 'string' ? info.license : info.license?.type || '';
    await record(
      'npm',
      info.name || pkgName,
      info.version || pkgVersion,
      licence,
      `https://registry.npmjs.org/${pkgName}/-/${pkgName.split('/').pop()}-${pkgVersion}.tgz`,
      pkgDir,
    );
  }
} else if (await exists('package-lock.json')) {
  const lock = await readJson('package-lock.json');
  for (const [location, p] of Object.entries(lock.packages)) {
    if (!location) continue;
    let installed;
    try {
      installed = await readJson(path.join(location, 'package.json'));
    } catch (error) {
      if (error.code === 'ENOENT' && p.optional) continue;
      throw error;
    }
    await record('npm', installed.name, p.version, p.license, p.resolved, path.join(root, location));
  }
}

await mkdir(output, { recursive: true });
await writeFile(
  path.join(output, 'manifest.json'),
  JSON.stringify(
    {
      target: 'x86_64-pc-windows-msvc',
      scope: 'Locked Windows graph, including build tools; installed frontend and development packages',
      packages: report,
    },
    null,
    2,
  ) + '\n',
);
await copyFile('THIRD_PARTY_NOTICES.md', path.join(output, 'README.md'));
await copyFile('LICENSE', path.join(output, 'SOLARA-LICENSE.txt'));
console.log(`Preserved notices for ${report.length} locked packages in ${path.relative(root, output)}`);
