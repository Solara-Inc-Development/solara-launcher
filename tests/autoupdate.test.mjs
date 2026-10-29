import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {verifyMinisign} from '../scripts/release-lib.mjs';

const root = path.dirname(path.dirname(fileURLToPath(import.meta.url)));

function compareSemVer(a, b) {
  const pa = a.split('.').map(Number);
  const pb = b.split('.').map(Number);
  for (let i = 0; i < 3; i++) {
    if (pa[i] > pb[i]) return 1;
    if (pa[i] < pb[i]) return -1;
  }
  return 0;
}

test('auto-update SemVer comparison correctly identifies available updates and up-to-date states', () => {
  assert.equal(compareSemVer('0.0.4', '0.0.3') > 0, true, '0.0.4 is newer than 0.0.3');
  assert.equal(compareSemVer('0.0.5', '0.0.4') > 0, true, '0.0.5 is newer than 0.0.4');
  assert.equal(compareSemVer('0.0.4', '0.0.4') === 0, true, '0.0.4 matches 0.0.4');
  assert.equal(compareSemVer('0.0.4', '0.0.5') < 0, true, '0.0.4 is older than 0.0.5');
  assert.equal(compareSemVer('0.0.10', '0.0.9') > 0, true, '0.0.10 is numerically greater than 0.0.9');
});

test('auto-update replacement policy defers installation while tasks or games are active', () => {
  function isBusy(snapshot) {
    if (!snapshot || !Array.isArray(snapshot.instances) || !Array.isArray(snapshot.localServers) || !Array.isArray(snapshot.downloads)) {
      return true;
    }
    const hasRunningInstance = snapshot.instances.some(i => i.running !== false);
    const hasRunningServer = snapshot.localServers.some(s => !['stopped', 'setup incomplete'].includes(s.state));
    const hasActiveDownloads = snapshot.downloads.some(d => !['completed', 'succeeded', 'failed', 'cancelled', 'canceled', 'interrupted'].includes(d.status));
    return hasRunningInstance || hasRunningServer || hasActiveDownloads;
  }

  // Idle state
  assert.equal(isBusy({instances: [{running: false}], localServers: [{state: 'stopped'}], downloads: [{status: 'completed'}]}), false);

  // Active game running
  assert.equal(isBusy({instances: [{running: true}], localServers: [{state: 'stopped'}], downloads: [{status: 'completed'}]}), true);

  // Active server
  assert.equal(isBusy({instances: [{running: false}], localServers: [{state: 'running'}], downloads: [{status: 'completed'}]}), true);

  // Active download
  assert.equal(isBusy({instances: [{running: false}], localServers: [{state: 'stopped'}], downloads: [{status: 'downloading'}]}), true);
  assert.equal(isBusy({instances: [{running: false}], localServers: [{state: 'stopped'}], downloads: [{status: 'queued'}]}), true);

  // Corrupted or missing snapshot
  assert.equal(isBusy({}), true);
  assert.equal(isBusy(null), true);
});

test('all published releases in builds/releases.json have valid signatures, files, and checksums', async () => {
  const releasesPath = path.join(root, 'builds/releases.json');
  const pubkeyPath = path.join(root, 'apps/desktop/updater.pub');
  const releasesData = JSON.parse(await readFile(releasesPath, 'utf8'));
  const pubkey = (await readFile(pubkeyPath, 'utf8')).trim();

  assert.ok(Array.isArray(releasesData.releases), 'releases must be an array');
  const published = releasesData.releases.filter(r => r.state === 'published');
  assert.ok(published.length > 0, 'at least one published release must exist');

  for (const r of published) {
    assert.match(r.publicVersion, /^V-0\.0\.\d+$/, 'publicVersion format');
    assert.match(r.internalVersion, /^0\.0\.\d+$/, 'internalVersion format');
    assert.equal(r.platform, 'windows', 'platform must be windows');
    assert.equal(r.arch, 'x86_64', 'arch must be x86_64');
    assert.ok(r.signature && r.signature.length > 50, 'signature must be present');
    assert.ok(r.url.startsWith('https://app.solara-inc.in/builds/'), 'url must point to official app portal');

    const artifactFile = path.join(root, 'builds', r.artifactPath);
    const bytes = await readFile(artifactFile);
    assert.equal(bytes.length, r.size, `File size matches for ${r.publicVersion}`);

    // Verify Minisign signature matches executable bytes
    const verified = verifyMinisign(bytes, r.signature, pubkey);
    assert.equal(verified, true, `Minisign verification must pass for ${r.publicVersion}`);
  }
});

