// Build notices from the locked Windows dependency graph; never invent grants.
import {execFileSync} from 'node:child_process';
import {readFile,writeFile,mkdir,readdir,copyFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
const root=process.cwd(),output=path.resolve(process.argv[2]||'dist/third-party');
const readJson=async file=>JSON.parse(await readFile(file,'utf8'));
const supplements=await readJson('third-party/licence-supplements/sources.json');
const metadata=JSON.parse(execFileSync('cargo',['metadata','--locked','--format-version','1','--filter-platform','x86_64-pc-windows-msvc'],{encoding:'utf8',maxBuffer:16*1024*1024}));
const ids=new Set(metadata.resolve.nodes.map(n=>n.id));
const retired=new Set(['theseus','ariadne','modrinth-content-management','path-util','serde-binhum','daedalus','async-minecraft-ping']);
if(metadata.packages.some(p=>retired.has(p.name)))throw Error('Retired engine dependency remains in the lockfile');
async function texts(dir,depth=0){
 const found=[];
 for(const entry of await readdir(dir,{withFileTypes:true})){
  if(entry.isSymbolicLink())continue;
  const file=path.join(dir,entry.name);
  if(entry.isDirectory()&&depth<2&&entry.name!=='node_modules')found.push(...await texts(file,depth+1));
  if(entry.isFile()&&/^(licen[sc]e|copying|notice|copyright|unlicense)($|[.\-_])/i.test(entry.name))found.push(file);
 }
 return found;
}
const report=[];
async function record(ecosystem,name,version,licence,source,dir){
 if(!licence||/UNLICENSED|SEE LICENSE/i.test(licence))throw Error(`Review required: ${name}@${version}`);
 const dest=path.join(output,ecosystem,name.replaceAll('/','_')+'-'+version);await mkdir(dest,{recursive:true});
 const files=await texts(dir);let supplement=null;
 // Published platform-tool archives share their parent package's licence.
 // Only use an installed parent of the exact same locked version.
 const parent=name.startsWith('@esbuild/')?'esbuild':name.startsWith('@rollup/rollup-')?'rollup':name.startsWith('@tauri-apps/cli-')?'@tauri-apps/cli':null;
 if(!files.length&&ecosystem==='npm'&&parent){
  const parentDir=path.join(root,'node_modules',parent),info=await readJson(path.join(parentDir,'package.json'));
  if(info.version!==version)throw Error('Platform tool and parent version mismatch');
  files.push(...await texts(parentDir));supplement={parentPackage:`${parent}@${version}`};
 }
 if(!files.length){
  supplement=supplements[`${name}@${version}`];
  if(!supplement)throw Error(`Missing licence text: ${name}@${version}`);
  const local=path.join(root,'third-party/licence-supplements',supplement.file);
  if(createHash('sha256').update(await readFile(local)).digest('hex')!==supplement.sha256)throw Error('Changed licence supplement');
  files.push(local);
 }
 const notices=[];
 for(const [index,file] of files.entries()){
  const filename=index+'-'+path.basename(file);await copyFile(file,path.join(dest,filename));notices.push(filename);
 }
 report.push({ecosystem,name,version,licence,source,notices,supplement});
}
for(const p of metadata.packages.filter(p=>ids.has(p.id)&&p.source)){
 await record('cargo',p.name,p.version,p.license,`https://crates.io/api/v1/crates/${p.name}/${p.version}/download`,path.dirname(p.manifest_path));
}
const lock=await readJson('package-lock.json');
for(const [location,p] of Object.entries(lock.packages)){
 if(!location)continue;
 let installed;
 try{installed=await readJson(path.join(location,'package.json'))}catch(error){if(error.code==='ENOENT'&&p.optional)continue;throw error}
 await record('npm',installed.name,p.version,p.license,p.resolved,path.join(root,location));
}
await writeFile(path.join(output,'manifest.json'),JSON.stringify({target:'x86_64-pc-windows-msvc',scope:'Locked Windows graph, including build tools; installed frontend and development packages',packages:report},null,2)+'\n');
await copyFile('THIRD_PARTY_NOTICES.md',path.join(output,'README.md'));
await copyFile('LICENSE',path.join(output,'SOLARA-LICENSE.txt'));
console.log(`Preserved notices for ${report.length} locked packages in ${path.relative(root,output)}`);
