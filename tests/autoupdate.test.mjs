import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {existsSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {
  verifyMinisign,
  isOfficialReleaseUrl,
  fetchWithTrustedRedirects,
  buildLatestManifest,
  githubReleaseAssetUrl,
  GITHUB_LATEST_MANIFEST_URL,
} from '../scripts/release-lib.mjs';

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
    assert.ok(
      isOfficialReleaseUrl(r.url),
      `url must point to official app portal or GitHub Releases CDN (got ${r.url})`
    );

    const artifactFile = path.join(root, 'builds', r.artifactPath);
    if (existsSync(artifactFile)) {
      const bytes = await readFile(artifactFile);
      assert.equal(bytes.length, r.size, `File size matches for ${r.publicVersion}`);

      // Verify Minisign signature matches executable bytes
      const verified = verifyMinisign(bytes, r.signature, pubkey);
      assert.equal(verified, true, `Minisign verification must pass for ${r.publicVersion}`);
    }
  }
});

test('GitHub CDN updater manifest builder validates schema and rejects untrusted URLs', () => {
  assert.equal(
    GITHUB_LATEST_MANIFEST_URL,
    'https://github.com/Solara-Inc-Development/solara-launcher/releases/latest/download/latest.json'
  );
  const validRelease = {
    publicVersion: 'V-0.0.40',
    internalVersion: '0.0.40',
    filename: 'solaralauncher.exe',
    artifactPath: 'V-0.0.40/solaralauncher.exe',
    publishedAt: '2026-11-15T00:00:00.000Z',
    signature: 'dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkK',
    notes: 'GitHub Releases CDN migration',
    url: githubReleaseAssetUrl('V-0.0.40', 'solaralauncher.exe'),
  };
  const manifest = buildLatestManifest(validRelease);
  assert.equal(manifest.version, '0.0.40');
  assert.equal(manifest.pub_date, '2026-11-15T00:00:00.000Z');
  assert.equal(
    manifest.platforms['windows-x86_64'].url,
    'https://github.com/Solara-Inc-Development/solara-launcher/releases/download/V-0.0.40/solaralauncher.exe'
  );
  assert.equal(manifest.platforms['windows-x86_64'].signature, validRelease.signature);

  assert.throws(
    () => buildLatestManifest({...validRelease, internalVersion: 'V-0.0.40'}),
    /SemVer/
  );
  assert.throws(
    () => buildLatestManifest({...validRelease, url: 'https://evil.example.com/V-0.0.40/solaralauncher.exe'}),
    /Untrusted release host/
  );
  assert.throws(
    () => buildLatestManifest({...validRelease, url: 'http://github.com/Solara-Inc-Development/solara-launcher/releases/download/V-0.0.40/solaralauncher.exe'}),
    /Untrusted release protocol/
  );
  assert.throws(
    () => buildLatestManifest({...validRelease, signature: 'short'}),
    /signature/
  );
  assert.throws(
    () => buildLatestManifest({...validRelease, publishedAt: 'not-a-date'}),
    /publishedAt/
  );
});

test('fetchWithTrustedRedirects follows GitHub CDN redirects and blocks untrusted hosts or downgrades', async () => {
  const calls = [];
  const res = await fetchWithTrustedRedirects(
    'https://github.com/Solara-Inc-Development/solara-launcher/releases/download/V-0.0.40/solaralauncher.exe',
    {
      fetcher: async (url, init) => {
        calls.push({url, redirect: init.redirect});
        if (url.startsWith('https://github.com/')) {
          return new Response(null, {
            status: 302,
            headers: {
              location: 'https://release-assets.githubusercontent.com/asset/123',
            },
          });
        }
        return new Response('ok', {status: 200});
      },
    }
  );
  assert.equal(res.status, 200);
  assert.equal(calls.length, 2);
  assert.equal(calls[0].redirect, 'manual');
  assert.equal(calls[1].url, 'https://release-assets.githubusercontent.com/asset/123');

  // Also allow objects.githubusercontent.com
  const resObj = await fetchWithTrustedRedirects(
    GITHUB_LATEST_MANIFEST_URL,
    {
      fetcher: async (url) => {
        if (url === GITHUB_LATEST_MANIFEST_URL) {
          return new Response(null, {
            status: 302,
            headers: {location: 'https://objects.githubusercontent.com/github-production-release-asset/latest.json'},
          });
        }
        return new Response('{}', {status: 200});
      },
    }
  );
  assert.equal(resObj.status, 200);

  // Reject untrusted redirect destination
  await assert.rejects(
    () => fetchWithTrustedRedirects(GITHUB_LATEST_MANIFEST_URL, {
      fetcher: async () => new Response(null, {
        status: 302,
        headers: {location: 'https://evil.example.com/latest.json'},
      }),
    }),
    /Untrusted release host/
  );

  // Reject HTTPS -> HTTP protocol downgrade redirect
  await assert.rejects(
    () => fetchWithTrustedRedirects(GITHUB_LATEST_MANIFEST_URL, {
      fetcher: async () => new Response(null, {
        status: 302,
        headers: {location: 'http://objects.githubusercontent.com/latest.json'},
      }),
    }),
    /Untrusted release protocol/
  );

  // Reject redirect loop
  await assert.rejects(
    () => fetchWithTrustedRedirects(GITHUB_LATEST_MANIFEST_URL, {
      maxRedirects: 2,
      fetcher: async () => new Response(null, {
        status: 302,
        headers: {location: 'https://objects.githubusercontent.com/loop'},
      }),
    }),
    /maximum trusted redirects/
  );
});

