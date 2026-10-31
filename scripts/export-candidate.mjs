// Export genuine Windows build outputs with exact corresponding source.
import {readFile,writeFile,mkdir,readdir,copyFile,stat} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {verifyPe,verifyApplication} from './release-lib.mjs';
const output=path.resolve(process.argv[2]||'');
if(!process.argv[2]||process.platform!=='win32')throw Error('Pass an output directory on the actual Windows runner');
const reservation=JSON.parse(await readFile('builds/current-build.json','utf8'));
const nsis='target/release/bundle/nsis';
const allFiles=(await readdir(nsis)).filter(f=>f.endsWith('.exe'));
const matching=allFiles.filter(f=>f.includes(reservation.internalVersion));
let installerName=matching[0];
if(!installerName&&allFiles.length){
  const stats=await Promise.all(allFiles.map(async f=>({f,mtime:(await stat(path.join(nsis,f))).mtimeMs})));
  stats.sort((a,b)=>b.mtime-a.mtime);
  installerName=stats[0].f;
}
if(!installerName)throw Error('Expected a genuine NSIS installer in '+nsis);
const installer=path.join(nsis,installerName),app='target/release/solaralauncher.exe';
verifyPe(await readFile(installer),[0x14c,0x8664]);const application=await verifyApplication(app);
await mkdir(path.join(output,'application'),{recursive:true});
await copyFile(installer,path.join(output,'solaralauncher.exe'));
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
await writeFile(path.join(output,'candidate.json'),JSON.stringify({...reservation,...application,sourceTree:tree,state:'unsigned-testing-candidate',acceptance:false,artifacts},null,2)+'\n');
await writeFile(path.join(output,'README.md'),`# Solara ${reservation.publicVersion} testing candidate\n\nBuilt from ${reservation.commit}. This is an acceptance-test candidate, not a stable release. The updater signature is added privately on the Solara build VPS before final staging. Windows publisher signing is not claimed.\n\nThe independent Minecraft core exposes the loaders declared by the preview. Microsoft sign-in needs an approved Solara application ID; actual login/play, installation and automatic upgrades still need testing. Hosted account services use the separate private deployment.\n\nThe NSIS installer and bundled x64 application are separate files, each named solaralauncher.exe. Exact corresponding source, version edits and lockfiles are in source.zip. Dependency notices are bundled in the app.\n`);
console.log(`Exported actual ${reservation.publicVersion} Windows candidate and corresponding source.`);
