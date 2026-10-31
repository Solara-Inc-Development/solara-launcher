// Publish the reviewed installer together with its exact CI-exported source.
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {isDownloadable,buildNumber,publicVersion} from '../packages/shared/domain.mjs';
import {verifyPe,verifyMinisign} from './release-lib.mjs';

export function validateRelease(release,version){
  const number=buildNumber(version);
  if(version!==publicVersion(number)||release?.publicVersion!==version||!isDownloadable(release))throw Error('Expected a published, verified canonical release');
  const prefix=`https://app.solara-inc.in/builds/${version}/`;
  if(release.url!==prefix+'solaralauncher.exe')throw Error('Unexpected official installer URL');
  const source=release.artifacts?.['source.zip'];
  if(!source||!Number.isSafeInteger(source.size)||source.size<100||!/^[a-f0-9]{64}$/.test(source.sha256||''))throw Error('Exact corresponding source size/hash is required');
  return prefix;
}

export function verifyReleaseAssets(release,version,assets,key){
  validateRelease(release,version);
  const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
  const exe=assets['solaralauncher.exe'],source=assets['source.zip'];
  if(exe.length!==release.size||sha(exe)!==release.sha256)throw Error('Installer bytes do not match the reviewed release');
  verifyPe(exe,[0x14c,0x8664]);
  const signature=assets['solaralauncher.exe.sig'].toString().trim();
  if(signature!==release.signature.trim())throw Error('Signature differs from the reviewed release');
  verifyMinisign(exe,signature,key,release.internalVersion);
  const sourceRecord=release.artifacts['source.zip'];
  if(source.length!==sourceRecord.size||sha(source)!==sourceRecord.sha256||source.readUInt32LE(0)!==0x04034b50)throw Error('Corresponding source archive does not match the CI export');
}

async function fetchBytes(url,limit){
  const response=await fetch(url,{signal:AbortSignal.timeout(60000),redirect:'error'});
  if(!response.ok)throw Error(`Official artifact returned HTTP ${response.status}`);
  if(Number(response.headers.get('content-length'))>limit)throw Error('Official artifact exceeds size limit');
  const chunks=[];let size=0;
  for await(const chunk of response.body){size+=chunk.length;if(size>limit)throw Error('Official artifact exceeds size limit');chunks.push(chunk)}
  return Buffer.concat(chunks);
}

async function prepare(version,output){
  if(!version)throw Error('Pass the exact release version');
  const bootstrap=JSON.parse((await fetchBytes('https://api.solara-inc.in/v1/launcher/bootstrap',5_000_000)).toString());
  const release=bootstrap.releases?.find(r=>r.publicVersion===version);
  const prefix=validateRelease(release,version);
  const assets={};
  for(const name of ['solaralauncher.exe','solaralauncher.exe.sig','source.zip'])assets[name]=await fetchBytes(prefix+name,name.endsWith('.sig')?16384:256_000_000);
  const root=path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  verifyReleaseAssets(release,version,assets,(await readFile(path.join(root,'apps/desktop/updater.pub'),'utf8')).trim());
  await mkdir(output,{recursive:true});
  for(const [name,bytes] of Object.entries(assets))await writeFile(path.join(output,name),bytes,{flag:'wx'});
  await writeFile(path.join(output,'release_notes.md'),`# Solara Launcher ${version}\n\n${release.notes}\n\nUpdater-signed Windows testing preview. Physical Windows install, game and update acceptance remain pending.\n`,{flag:'wx'});
  console.log(`Verified ${version}: installer signature/version, SHA-256 and exact CI source archive.`);
}

if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  prepare(process.argv[2],path.resolve(process.argv[3]||'release-assets')).catch(error=>{console.error(error.message);process.exitCode=1});
}
