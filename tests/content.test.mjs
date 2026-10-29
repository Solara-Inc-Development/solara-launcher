import test from 'node:test';
import assert from 'node:assert/strict';
import {providerId, providerIcon, searchUrl, projectUrl, normalizeHits, normalizeVersions} from '../packages/shared/content.mjs';
test('provider URLs bind the result kind and reject invalid IDs and remote icons', () => {
  const url = new URL(searchUrl('a & b', 'modpack'));
  assert.equal(url.searchParams.get('query'), 'a & b');
  assert.deepEqual(JSON.parse(url.searchParams.get('facets')), [['project_type:modpack']]);
  assert.equal(projectUrl({project_type:'modpack', slug:'my-pack'}), 'https://modrinth.com/modpack/my-pack');
  for (const id of ['../mod', 'id?token=x', '', 'x'.repeat(101)]) assert.throws(() => providerId(id));
  assert.equal(providerIcon('https://cdn.modrinth.com/data/abc/icon.png'), 'https://cdn.modrinth.com/data/abc/icon.png');
  for (const url of ['javascript:alert(1)', 'https://cdn.modrinth.com.evil.test/icon.png', 'https://secret@cdn.modrinth.com/a', 'http://cdn.modrinth.com/a']) assert.equal(providerIcon(url), null);
});
test('search metadata stays tied to its returned kind and is bounded', () => {
  const hits = normalizeHits({hits:[{project_id:'pack123',project_type:'modpack',title:'Pack',slug:'pack',downloads:42},{project_id:'mod123',project_type:'mod',title:'Other type'}]},'modpack');
  assert.equal(hits.length,1);assert.equal(hits[0].project_type,'modpack');assert.equal(hits[0].downloads,42);
  assert.throws(() => normalizeHits({hits:null},'mod'));
});
test('modpack versions require matching project and mrpack files and sort by actual publication date', () => {
  const make = (id,type,date,overrides={}) => ({id,project_id:'pack123',name:id,version_type:type,date_published:date,files:[{filename:'pack.mrpack'}],game_versions:['1.21.1'],loaders:['fabric'],...overrides});
  const versions=normalizeVersions([make('old','release','2025-01-01'),make('new','beta','2026-01-01'),make('foreign','release','2026-01-02',{project_id:'other'}),make('notpack','release','2026-01-02',{files:[{filename:'mod.jar'}]})],'pack123');
  assert.deepEqual(versions.map(v=>v.id),['new','old']);assert.equal(versions[1].type,'release');assert.equal(versions[0].games,'1.21.1');
});
