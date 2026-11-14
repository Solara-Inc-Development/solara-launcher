import {createHash,createPublicKey,verify} from 'node:crypto';
import {readFile,writeFile,mkdir,open,rename,unlink,readdir} from 'node:fs/promises';
import path from 'node:path';
import {publicVersion,buildNumber,releaseArtifactPath} from '../packages/shared/domain.mjs';

export const GITHUB_REPO_SLUG='Solara-Inc-Development/solara-launcher';
export const GITHUB_LATEST_MANIFEST_URL=`https://github.com/${GITHUB_REPO_SLUG}/releases/latest/download/latest.json`;
export const TRUSTED_RELEASE_HOSTS=new Set([
 'github.com',
 'objects.githubusercontent.com',
 'release-assets.githubusercontent.com',
 'github-releases.githubusercontent.com',
 'codeload.github.com',
 'app.solara-inc.in',
 'api.solara-inc.in',
]);

export function githubReleaseAssetUrl(version,filename='solaralauncher.exe'){
 return `https://github.com/${GITHUB_REPO_SLUG}/releases/download/${version}/${filename}`;
}

export function isTrustedReleaseHost(rawUrl,allowedHosts=TRUSTED_RELEASE_HOSTS){
 let parsed;
 try{parsed=new URL(rawUrl);}catch{throw Error('Invalid release URL');}
 if(parsed.protocol!=='https:'||parsed.username||parsed.password){
  throw Error(`Untrusted release protocol or credentials in URL: ${rawUrl}`);
 }
 if(!allowedHosts.has(parsed.hostname)){
  throw Error(`Untrusted release host: ${parsed.hostname}`);
 }
 return parsed;
}

export function isOfficialReleaseUrl(rawUrl){
 try{
  const u=isTrustedReleaseHost(rawUrl);
  if(u.hostname==='app.solara-inc.in')return u.pathname.startsWith('/builds/');
  if(u.hostname==='github.com')return u.pathname.startsWith(`/${GITHUB_REPO_SLUG}/releases/download/`);
  return false;
 }catch{
  return false;
 }
}

export async function fetchWithTrustedRedirects(rawUrl,{method='GET',timeoutMs=60000,maxRedirects=5,allowedHosts=TRUSTED_RELEASE_HOSTS,fetcher=globalThis.fetch}={}){
 let currentUrl=isTrustedReleaseHost(rawUrl,allowedHosts).href;
 for(let hop=0;hop<=maxRedirects;hop++){
  const response=await fetcher(currentUrl,{method,signal:AbortSignal.timeout(timeoutMs),redirect:'manual'});
  if([301,302,303,307,308].includes(response.status)){
   const location=response.headers.get('location');
   await response.body?.cancel?.().catch(()=>{});
   if(!location)throw Error(`Redirect response (${response.status}) missing location header`);
   const nextUrl=new URL(location,currentUrl).href;
   currentUrl=isTrustedReleaseHost(nextUrl,allowedHosts).href;
   continue;
  }
  return response;
 }
 throw Error(`Exceeded maximum trusted redirects (${maxRedirects}) for ${rawUrl}`);
}

export function buildLatestManifest(release){
 if(!release||typeof release!=='object')throw Error('Release metadata is required for latest.json');
 if(!/^\d+\.\d+\.\d+$/.test(release.internalVersion||''))throw Error('Valid numeric SemVer internalVersion is required for latest.json');
 if(!release.publishedAt||Number.isNaN(Date.parse(release.publishedAt)))throw Error('Valid RFC3339 publishedAt timestamp is required for latest.json');
 const sig=(release.signature||'').trim();
 if(sig.length<=50)throw Error('Valid Minisign signature is required for latest.json');
 const parsed=isTrustedReleaseHost(release.url);
 const expectedRel=releaseArtifactPath(release);
 if(!parsed.pathname.endsWith('/'+expectedRel))throw Error('Release URL path does not match artifactPath');
 return {
  version:release.internalVersion,
  notes:typeof release.notes==='string'?release.notes:'',
  pub_date:release.publishedAt,
  platforms:{
   'windows-x86_64':{
    signature:sig,
    url:parsed.href,
   },
  },
 };
}
export async function reserveBuild(dir,{commit='unknown',minimum=1}={}){
 await mkdir(dir,{recursive:true});const lock=path.join(dir,'.allocation.lock');const guard=await open(lock,'wx').catch(()=>{throw Error('Another build is allocating a version; retry after it completes')});
 try{let registry={schemaVersion:1,allocations:[]};try{registry=JSON.parse(await readFile(path.join(dir,'allocations.json'),'utf8'))}catch(e){if(e.code!=='ENOENT')throw e}
 let releases=[];try{releases=JSON.parse(await readFile(path.join(dir,'releases.json'),'utf8')).releases||[]}catch(e){if(e.code!=='ENOENT')throw e}
 const existing=await readdir(dir);const consumed=existing.flatMap(name=>{const m=/^(V-0\.0\.\d+)\.reservation\.json$/.exec(name);return m?[buildNumber(m[1])+1]:[]});
 let n=Math.max(minimum,...consumed,...registry.allocations.map(a=>buildNumber(a.publicVersion)+1),...releases.map(r=>buildNumber(r.publicVersion)+1));if(!Number.isSafeInteger(n))throw Error('Invalid build counter');
 const reservation={buildId:crypto.randomUUID(),publicVersion:publicVersion(n),internalVersion:`0.0.${n}`,filename:'solaralauncher.exe',artifactPath:`${publicVersion(n)}/solaralauncher.exe`,commit,allocatedAt:new Date().toISOString(),state:'allocated'};
 await writeFile(path.join(dir,reservation.publicVersion+'.reservation.json'),JSON.stringify(reservation,null,2),{flag:'wx'});registry.allocations.push(reservation);const temp=path.join(dir,'.allocations.'+reservation.buildId+'.tmp');await writeFile(temp,JSON.stringify(registry,null,2));await rename(temp,path.join(dir,'allocations.json'));return reservation;
 }finally{await guard.close();await unlink(lock)}
}
export async function immutableCopy(source,target){const incoming=await readFile(source);let fd;try{fd=await open(target,'wx');await fd.writeFile(incoming);await fd.sync();}catch(e){if(e.code!=='EEXIST')throw e;const old=await readFile(target);if(!old.equals(incoming))throw Error('Refusing to replace an immutable build with different bytes');}finally{await fd?.close()};return incoming;}
export function verifyMinisign(artifact,encodedSignature,encodedPublicKey,expectedVersion=null){
 const keyText=Buffer.from(encodedPublicKey.trim(),'base64').toString('utf8');const keyLines=keyText.trim().split(/\r?\n/);const key=Buffer.from(keyLines.at(-1)||'','base64');
 const signatureText=Buffer.from(encodedSignature.trim(),'base64').toString('utf8');const lines=signatureText.trim().split(/\r?\n/);if(lines.length!==4||!lines[0].startsWith('untrusted comment:')||!lines[2].startsWith('trusted comment: '))throw Error('Invalid Tauri signature format');
 const sig=Buffer.from(lines[1],'base64'),global=Buffer.from(lines[3],'base64');if(key.length!==42||sig.length!==74||global.length!==64||!key.subarray(2,10).equals(sig.subarray(2,10)))throw Error('Signature key mismatch');
 const publicKey=createPublicKey({key:Buffer.concat([Buffer.from('302a300506032b6570032100','hex'),key.subarray(10)]),format:'der',type:'spki'});
 const alg=sig.subarray(0,2).toString();if(!['Ed','ED'].includes(alg))throw Error('Unsupported signature algorithm');const content=alg==='ED'?createHash('blake2b512').update(artifact).digest():artifact;
 if(!verify(null,content,publicKey,sig.subarray(10)))throw Error('Installer signature verification failed');
 const comment=Buffer.from(lines[2].slice('trusted comment: '.length));if(!verify(null,Buffer.concat([sig.subarray(10),comment]),publicKey,global))throw Error('Trusted signature metadata verification failed');if(expectedVersion && !comment.toString().split('\t').includes('version:'+expectedVersion))throw Error('Signed version does not match release metadata');return true;
}
export function verifyPe(bytes,allowedMachines=[0x8664]){
 if(bytes.length<1024||bytes.subarray(0,2).toString()!=='MZ')throw Error('Artifact is not a Windows executable');
 const offset=bytes.readUInt32LE(0x3c);
 if(offset<64||offset+24>bytes.length||bytes.subarray(offset,offset+4).toString('binary')!=='PE\x00\x00')throw Error('Invalid Windows PE header');
 const machine=bytes.readUInt16LE(offset+4),sections=bytes.readUInt16LE(offset+6),optionalSize=bytes.readUInt16LE(offset+20),flags=bytes.readUInt16LE(offset+22);
 if(!allowedMachines.includes(machine))throw Error('Unexpected Windows executable architecture');
 const expectedMagic=machine===0x8664?0x20b:0x10b;
 if(!sections||sections>96||optionalSize<(machine===0x8664?112:96)||offset+24+optionalSize+40*sections>bytes.length||bytes.readUInt16LE(offset+24)!==expectedMagic||!(flags&0x2)||(flags&0x2000))throw Error('Invalid Windows executable structure');
 return true;
}
export async function verifyApplication(file){
 const bytes=await readFile(file);verifyPe(bytes);
 return {applicationSha256:createHash('sha256').update(bytes).digest('hex'),applicationSize:bytes.length};
}
export async function verifyArtifact(file,signature,key,version=null){
 const bytes=await readFile(file);
 // NSIS uses an x86 bootstrapper even when the bundled application is x64.
 // The application's architecture is verified separately when staging.
 verifyPe(bytes,[0x14c,0x8664]);verifyMinisign(bytes,signature,key,version);
 return {size:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex')};
}
