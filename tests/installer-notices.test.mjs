import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp, mkdir, writeFile, cp, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {verifyNoticeTree} from '../scripts/installer-notices.mjs';

async function fixture(fn) {
  const directory = await mkdtemp(path.join(tmpdir(),'solara-notice-test-'));
  const expected = path.join(directory,'expected'), actual = path.join(directory,'actual');
  const relative = 'npm/example-1.0.0/LICENSE';
  await mkdir(path.dirname(path.join(expected,relative)),{recursive:true});
  await writeFile(path.join(expected,relative),'Example dependency copyright and licence text.');
  await writeFile(path.join(expected,'manifest.json'),JSON.stringify({target:'x86_64-pc-windows-msvc',packages:[{ecosystem:'npm',name:'example',version:'1.0.0',notices:['LICENSE']}]}));
  await cp(expected,actual,{recursive:true});
  try { await fn({expected,actual,relative}); } finally {await rm(directory,{recursive:true,force:true});}
}

test('installer notice validation checks the actual bundled licence bytes', async () => fixture(async ({expected,actual}) => {
  assert.deepEqual(await verifyNoticeTree(actual,expected),{packages:1,files:1,verified:true});
}));
test('installer notice validation rejects missing or altered licence files', async () => fixture(async ({expected,actual,relative}) => {
  await writeFile(path.join(actual,relative),'Changed');
  await assert.rejects(verifyNoticeTree(actual,expected),/licence differs/);
  await rm(path.join(actual,relative));
  await assert.rejects(verifyNoticeTree(actual,expected),/ENOENT/);
}));
test('installer notice validation rejects a substituted manifest', async () => fixture(async ({expected,actual}) => {
  await writeFile(path.join(actual,'manifest.json'),'{}');
  await assert.rejects(verifyNoticeTree(actual,expected),/manifest differs/);
}));
