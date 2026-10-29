import {createHash,createPublicKey,verify} from 'node:crypto';
import {readFile,writeFile,mkdir,open,rename,unlink,readdir} from 'node:fs/promises';
import path from 'node:path';
import {publicVersion,buildNumber} from '../packages/shared/domain.mjs';
export async function reserveBuild(dir,{commit='unknown',minimum=1}={}){
 await mkdir(dir,{recursive:true});const lock=path.join(dir,'.allocation.lock');const guard=await open(lock,'wx').catch(()=>{throw Error('Another build is allocating a version; retry after it completes')});
 try{let registry={schemaVersion:1,allocations:[]};try{registry=JSON.parse(await readFile(path.join(dir,'allocations.json'),'utf8'))}catch(e){if(e.code!=='ENOENT')throw e}
 const existing=await readdir(dir);const consumed=existing.flatMap(name=>{const m=/^(V-0\.0\.\d+)\.reservation\.json$/.exec(name);return m?[buildNumber(m[1])+1]:[]});
 let n=Math.max(minimum,...consumed,...registry.allocations.map(a=>buildNumber(a.publicVersion)+1));if(!Number.isSafeInteger(n))throw Error('Invalid build counter');
 const reservation={buildId:crypto.randomUUID(),publicVersion:publicVersion(n),internalVersion:`0.0.${n}`,filename:`Solara-Launcher-${publicVersion(n)}.exe`,commit,allocatedAt:new Date().toISOString(),state:'allocated'};
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
