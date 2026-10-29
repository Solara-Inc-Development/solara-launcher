import {readFile,writeFile,readdir,mkdir} from 'node:fs/promises';import {execFileSync} from 'node:child_process';import path from 'node:path';import {fileURLToPath} from 'node:url';import {reserveBuild,immutableCopy,verifyArtifact} from './release-lib.mjs';
const root=path.dirname(path.dirname(fileURLToPath(import.meta.url))),dir=path.join(root,'builds');
const mode=process.argv[2];const run=(bin,args,opts={})=>execFileSync(bin,args,{cwd:root,stdio:'inherit',...opts});
const readJson=async p=>JSON.parse(await readFile(p,'utf8'));const json=async(p,d)=>writeFile(p,JSON.stringify(d,null,2)+'\n');
if(mode==='reserve'){
 let minimum=1;let commit='uncommitted';try{commit=execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim()}catch{}
 if(process.argv.includes('--github')){const tags=JSON.parse(execFileSync('gh',['api','--paginate','--slurp','repos/{owner}/{repo}/releases','--jq','map(.[] | .tag_name)'],{cwd:root,encoding:'utf8'}));minimum=Math.max(1,...tags.filter(t=>/^solara-build-\d+$/.test(t)).map(t=>Number(t.slice(13))+1));}
 const r=await reserveBuild(dir,{commit,minimum});if(process.argv.includes('--github'))run('gh',['release','create','solara-build-'+Number(r.internalVersion.split('.').at(-1)),'--draft','--title',r.publicVersion,'--notes','Reserved immutable Solara build. Pending packaging, verification and publication.']);
 await json(path.join(dir,'current-build.json'),r);await json(path.join(root,'packages/shared/build.json'),{...r,channel:'release-candidate'});
 const config=await readJson(path.join(root,'apps/desktop/tauri.conf.json'));config.version=r.internalVersion;await json(path.join(root,'apps/desktop/tauri.conf.json'),config);
 const cargo=path.join(root,'apps/desktop/Cargo.toml');await writeFile(cargo,(await readFile(cargo,'utf8')).replace(/version = "[^"]+"/,`version = "${r.internalVersion}"`));console.log(`${r.publicVersion} allocated; never reuse if this build fails.`);
}else if(mode==='package'){
 if(process.platform!=='win32')throw Error('Windows packaging requires a Windows x64 runner with MSVC, WebView2 and NSIS prerequisites');
 const review=await readJson(path.join(root,'docs/upstream-license-review.json'));if(review.status!=='cleared'||!review.evidence?.length)throw Error('Upstream dependency licence review is unresolved; see THIRD_PARTY_NOTICES.md');
 if(!process.env.TAURI_SIGNING_PRIVATE_KEY||!process.env.SOLARA_UPDATER_PUBLIC_KEY)throw Error('Updater signing private key and trusted public key are required');
 await readJson(path.join(dir,'current-build.json'));run('npx',['tauri','build','--config','apps/desktop/tauri.conf.json','--bundles','nsis']);
}else if(mode==='stage'){
 const r=await readJson(path.join(dir,'current-build.json'));const source=process.argv[3];if(!source)throw Error('Pass the actual NSIS installer path');const sig=(await readFile(source+'.sig','utf8')).trim();const key=process.env.SOLARA_UPDATER_PUBLIC_KEY;if(!key)throw Error('Trusted public key is required');
 const checked=await verifyArtifact(source,sig,key,r.internalVersion);await immutableCopy(source,path.join(dir,r.filename));await immutableCopy(source+'.sig',path.join(dir,r.filename+'.sig'));
 const notesPath=process.argv[4];if(!notesPath)throw Error('Pass a release notes file');const notes=await readFile(notesPath,'utf8');if(!notes.trim())throw Error('Release notes are required');
 const metadata={...r,...checked,platform:'windows',arch:'x86_64',publishedAt:null,verifiedAt:new Date().toISOString(),signature:sig,notes,url:`https://launcher.solara-inc.in/builds/${r.filename}`,state:'staged'};await json(path.join(dir,r.publicVersion+'.json'),metadata);console.log(`${r.filename} staged, checksum and updater signature verified.`);
}else if(mode==='publish-manifest'){
 const version=process.argv[3];if(!version)throw Error('Pass the staged public version');const r=await readJson(path.join(dir,version+'.json'));await verifyArtifact(path.join(dir,r.filename),r.signature,process.env.SOLARA_UPDATER_PUBLIC_KEY||'',r.internalVersion);
 const response=await fetch(r.url,{signal:AbortSignal.timeout(30000)});if(!response.ok)throw Error('Uploaded installer URL is not publicly reachable');const remote=Buffer.from(await response.arrayBuffer());const local=await readFile(path.join(dir,r.filename));if(!remote.equals(local))throw Error('Public artifact bytes do not match verified local artifact');
 const archive=await readJson(path.join(dir,'releases.json'));if(archive.releases.some(x=>x.publicVersion===r.publicVersion))throw Error('Release already exists; use withdrawal for revoked versions');r.state='published';r.publishedAt=new Date().toISOString();archive.releases.push(r);await json(path.join(dir,'releases.json'),archive);console.log('Published metadata only after public artifact verification.');
}else if(mode==='withdraw'){
 const version=process.argv[3],reason=process.argv[4];if(!reason)throw Error('Pass public version and withdrawal reason');const archive=await readJson(path.join(dir,'releases.json'));const r=archive.releases.find(x=>x.publicVersion===version);if(!r)throw Error('Release not found');r.state='withdrawn';r.withdrawnAt=new Date().toISOString();r.withdrawalReason=reason;await json(path.join(dir,'releases.json'),archive);console.log('Release withdrawn; immutable artifact preserved.');
}else{throw Error('Use reserve [--github], package, stage <installer> <notes>, publish-manifest <version>, or withdraw <version> <reason>');}
