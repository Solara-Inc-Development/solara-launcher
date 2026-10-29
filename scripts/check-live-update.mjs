#!/usr/bin/env node
import assert from 'node:assert/strict';

async function checkLiveUpdate() {
  console.log('Checking live auto-update endpoint on api.solara-inc.in...');
  const res = await fetch('https://api.solara-inc.in/v1/launcher/update/windows/x86_64', {
    signal: AbortSignal.timeout(10000)
  });
  assert.equal(res.status, 200, 'API returns HTTP 200');
  const data = await res.json();
  console.log(`Current published version: ${data.version} (published: ${data.pub_date})`);
  assert.ok(data.version, 'must have version');
  assert.ok(data.pub_date, 'must have pub_date');
  assert.ok(data.platforms['windows-x86_64'], 'must have windows-x86_64 platform');
  const {url, signature} = data.platforms['windows-x86_64'];
  assert.ok(url.startsWith('https://'), 'url must be https');
  assert.ok(signature.length > 50, 'signature must be valid length');

  console.log(`Checking download URL: ${url}`);
  const head = await fetch(url, {method: 'HEAD', signal: AbortSignal.timeout(10000)});
  assert.equal(head.status, 200, 'download URL returns HTTP 200');
  const size = Number(head.headers.get('content-length'));
  console.log(`Installer payload size: ${(size / 1024 / 1024).toFixed(2)} MB`);
  assert.ok(size > 5_000_000, 'file size > 5MB');
  console.log('PASS: Live auto-update endpoint is healthy and verified!');
}

checkLiveUpdate().catch(e => {
  console.error('FAIL: Live update check failed:', e);
  process.exitCode = 1;
});
