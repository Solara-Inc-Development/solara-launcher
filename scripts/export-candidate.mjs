// Export genuine Windows standalone x64 build output with exact corresponding source.
import {readFile,writeFile,mkdir,copyFile} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {verifyApplication} from './release-lib.mjs';
import {verifyNoticeTree} from './installer-notices.mjs';
const output=path.resolve(process.argv[2]||'');
if(!process.argv[2]||process.platform!=='win32')throw Error('Pass an output directory on the actual Windows runner');
const reservation=JSON.parse(await readFile('builds/current-build.json','utf8'));
const app='target/release/solaralauncher.exe';
const application=await verifyApplication(app);
const bundledNotices=await verifyNoticeTree(path.resolve('dist/third-party'),path.resolve('dist/third-party'));
await mkdir(path.join(output,'application'),{recursive:true});
await copyFile(app,path.join(output,'solaralauncher.exe'));
await copyFile(app,path.join(output,'application/solaralauncher.exe'));
await copyFile('CHANGELOG.md',path.join(output,'CHANGELOG.md'));
await copyFile('LICENSE',path.join(output,'LICENSE'));
await copyFile('THIRD_PARTY_NOTICES.md',path.join(output,'THIRD_PARTY_NOTICES.md'));
// Git archive of the index includes generated version/lock edits, never ignored
// prompts, local handoff, signing identity, private backend or environment files.
execFileSync('git',['add','-u']);
const tree=execFileSync('git',['write-tree'],{encoding:'utf8'}).trim();
execFileSync('git',['archive','--format=zip',`--output=${path.join(output,'source.zip')}`,tree]);
const artifacts={};
for(const file of ['solaralauncher.exe','application/solaralauncher.exe','source.zip']){
 const bytes=await readFile(path.join(output,file));artifacts[file]={size:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex')};
}
await writeFile(path.join(output,'candidate.json'),JSON.stringify({...reservation,...application,sourceTree:tree,state:'unsigned-testing-candidate',acceptance:false,distributionMode:'standalone-x64',artifacts,bundledNotices},null,2)+'\n');
await writeFile(path.join(output,'README.md'),`# Solara ${reservation.publicVersion} testing candidate\n\nBuilt from ${reservation.commit}. This is a direct standalone Windows x64 executable (no NSIS installer wrapper). The updater signature is added privately on the Solara build VPS before final staging. Windows publisher signing is not claimed.\n\nThe current launcher retains the upstream-derived engine and its required attribution. Actual Windows login/play and automatic in-place self-updates still need testing.\n\nExact corresponding source, version edits and lockfiles are in source.zip. GPL-3.0 licence and third-party dependency notices are embedded in the binary and included alongside the release.\n`);
console.log(`Exported actual ${reservation.publicVersion} standalone Windows x64 candidate and corresponding source.`);
