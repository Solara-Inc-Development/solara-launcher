import {readFile, readdir, mkdtemp, rm} from 'node:fs/promises';
import {existsSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {tmpdir} from 'node:os';
import path from 'node:path';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const windowsSevenZip = path.join(process.env.ProgramFiles || 'C:\\Program Files', '7-Zip', '7z.exe');
const defaultSevenZip = process.env.SOLARA_7ZIP_PATH || (process.platform === 'win32' && existsSync(windowsSevenZip) ? windowsSevenZip : '7z');

export async function verifyNoticeTree(actual, expected) {
  const manifest = await readFile(path.join(expected, 'manifest.json'));
  if (!(await readFile(path.join(actual, 'manifest.json'))).equals(manifest)) throw Error('Bundled dependency manifest differs from generated inventory');
  const inventory = JSON.parse(manifest);
  if (!inventory.packages?.length || inventory.target !== 'x86_64-pc-windows-msvc') throw Error('Invalid Windows notice inventory');
  let files = 0;
  for (const pkg of inventory.packages) {
    for (const notice of pkg.notices) {
      const relative = path.join(pkg.ecosystem, `${pkg.name.replaceAll('/', '_')}-${pkg.version}`, notice);
      const [bundled, generated] = await Promise.all([readFile(path.join(actual, relative)), readFile(path.join(expected, relative))]);
      if (sha(bundled) !== sha(generated)) throw Error(`Bundled licence differs: ${pkg.name}/${notice}`);
      files++;
    }
  }
  return {packages: inventory.packages.length, files, verified: true};
}

async function findInventory(directory) {
  for (const entry of await readdir(directory, {withFileTypes:true})) {
    const file = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      const found = await findInventory(file);
      if (found) return found;
    } else if (file.replaceAll('\\', '/').endsWith('/licenses/dependencies/manifest.json')) return path.dirname(file);
  }
  return null;
}

export async function verifyBundledNotices(installer, expected, sevenZip = defaultSevenZip) {
  const directory = await mkdtemp(path.join(tmpdir(), 'solara-installer-notices-'));
  try {
    execFileSync(sevenZip, ['x', '-y', `-o${directory}`, installer], {stdio:'pipe', maxBuffer:8*1024*1024});
    const inventory = await findInventory(directory);
    if (!inventory) throw Error('Installer is missing bundled dependency notices');
    return await verifyNoticeTree(inventory, expected);
  } finally {
    await rm(directory, {recursive:true, force:true});
  }
}
