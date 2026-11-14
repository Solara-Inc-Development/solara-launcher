#!/usr/bin/env node
import assert from 'node:assert/strict';
import {
  GITHUB_LATEST_MANIFEST_URL,
  fetchWithTrustedRedirects,
} from './release-lib.mjs';

async function verifyManifestEndpoint(name, endpointUrl, expectedUrlPrefix) {
  console.log(`Checking ${name}: ${endpointUrl}`);
  const res = await fetchWithTrustedRedirects(endpointUrl, { timeoutMs: 15000 });
  assert.equal(res.status, 200, `${name} returns HTTP 200`);
  const data = await res.json();
  console.log(`  version: ${data.version} (pub_date: ${data.pub_date})`);
  assert.match(String(data.version || ''), /^\d+\.\d+\.\d+$/, `${name} version must be numeric SemVer`);
  assert.ok(data.pub_date && !Number.isNaN(Date.parse(data.pub_date)), `${name} must have valid pub_date`);
  assert.ok(data.platforms?.['windows-x86_64'], `${name} must have windows-x86_64 platform`);
  const { url, signature } = data.platforms['windows-x86_64'];
  assert.ok(url.startsWith(expectedUrlPrefix), `${name} url must start with ${expectedUrlPrefix}, got ${url}`);
  assert.ok(typeof signature === 'string' && signature.trim().length > 50, `${name} signature must be valid length`);

  console.log(`  HEAD ${url}`);
  const head = await fetchWithTrustedRedirects(url, { method: 'HEAD', timeoutMs: 15000 });
  assert.equal(head.status, 200, `${name} download URL returns HTTP 200`);
  const size = Number(head.headers.get('content-length'));
  console.log(`  Installer payload size: ${(size / 1024 / 1024).toFixed(2)} MB`);
  assert.ok(size > 5_000_000, `${name} file size > 5MB`);
  return data;
}

async function checkLiveUpdate() {
  const gh = await verifyManifestEndpoint(
    'GitHub Releases CDN latest.json',
    GITHUB_LATEST_MANIFEST_URL,
    'https://github.com/Solara-Inc-Development/solara-launcher/releases/download/',
  );
  const vps = await verifyManifestEndpoint(
    'VPS transition bootstrap endpoint',
    'https://api.solara-inc.in/v1/launcher/update/windows/x86_64',
    'https://github.com/Solara-Inc-Development/solara-launcher/releases/download/',
  );
  assert.equal(gh.version, vps.version, 'GitHub CDN and VPS transition endpoint versions must match');
  console.log('PASS: Live GitHub Releases CDN auto-updater and transition endpoint are healthy and verified!');
}

checkLiveUpdate().catch(e => {
  console.error('FAIL: Live update check failed:', e);
  process.exitCode = 1;
});
