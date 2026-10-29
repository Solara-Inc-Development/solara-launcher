import {test} from 'node:test';import assert from 'node:assert/strict';import {mkdtemp,readFile,writeFile,rm} from 'node:fs/promises';import os from 'node:os';import path from 'node:path';import {generateKeyPairSync,sign,createHash} from 'node:crypto';import {reserveBuild,immutableCopy,verifyMinisign,verifyPe} from '../scripts/release-lib.mjs';
import {verifyArtifact,verifyApplication} from '../scripts/release-lib.mjs';

// In-memory header fixtures only; these are not runnable/distributable executables.
function peHeader(machine){
 const bytes=Buffer.alloc(1024),offset=128;
 bytes.write('MZ');bytes.writeUInt32LE(offset,0x3c);bytes.write('PE\0\0',offset);
 bytes.writeUInt16LE(machine,offset+4);bytes.writeUInt16LE(1,offset+6);
 bytes.writeUInt16LE(240,offset+20);bytes.writeUInt16LE(2,offset+22);
 bytes.writeUInt16LE(machine===0x8664?0x20b:0x10b,offset+24);
 return bytes;
}
function signatureFor(bytes,version){
 const {publicKey,privateKey}=generateKeyPairSync('ed25519');
 const id=Buffer.alloc(8,3),signature=sign(null,createHash('blake2b512').update(bytes).digest(),privateKey);
 const comment=`timestamp:123\tfile:fixture.bin\tversion:${version}`;
 const global=sign(null,Buffer.concat([signature,Buffer.from(comment)]),privateKey);
 return {
  signature:Buffer.from(`untrusted comment: test\n${Buffer.concat([Buffer.from('ED'),id,signature]).toString('base64')}\ntrusted comment: ${comment}\n${global.toString('base64')}\n`).toString('base64'),
  key:Buffer.from(`untrusted comment: test\n${Buffer.concat([Buffer.from('Ed'),id,publicKey.export({format:'der',type:'spki'}).subarray(-32)]).toString('base64')}\n`).toString('base64')
 };
}
test('NSIS x86 headers are distinct from the required x64 application and malformed PE files fail',()=>{
 const x86=peHeader(0x14c),x64=peHeader(0x8664);
 assert.equal(verifyPe(x86,[0x14c,0x8664]),true);
 assert.throws(()=>verifyPe(x86),/architecture/);
 assert.equal(verifyPe(x64),true);
 assert.throws(()=>verifyPe(peHeader(0xaa64),[0x14c,0x8664]),/architecture/);
 const truncated=Buffer.from(x64);truncated.writeUInt32LE(1010,0x3c);
 assert.throws(()=>verifyPe(truncated),/header/);
 for(const [at,value] of [[128+6,96],[128+20,0],[128+22,0x2002],[128+24,0x10b]]){
  const malformed=Buffer.from(x64);malformed.writeUInt16LE(value,at);
  assert.throws(()=>verifyPe(malformed),/structure/);
 }
});
test('signed x86 installer verification binds version and bytes while application verification requires x64',async()=>{
 const dir=await mkdtemp(path.join(os.tmpdir(),'solara-pe-test-'));
 try{
  const file=path.join(dir,'installer-header.bin'),app=path.join(dir,'application-header.bin');
  const bytes=peHeader(0x14c),signed=signatureFor(bytes,'0.0.1');
  await writeFile(file,bytes);await writeFile(app,peHeader(0x8664));
  assert.equal((await verifyArtifact(file,signed.signature,signed.key,'0.0.1')).size,1024);
  assert.equal((await verifyApplication(app)).applicationSize,1024);
  await assert.rejects(()=>verifyApplication(file),/architecture/);
  await assert.rejects(()=>verifyArtifact(file,signed.signature,signed.key,'0.0.2'),/version/);
  bytes[1000]=1;await writeFile(file,bytes);
  await assert.rejects(()=>verifyArtifact(file,signed.signature,signed.key,'0.0.1'),/verification failed/);
 }finally{await rm(dir,{recursive:true,force:true})}
});
test('allocation preserves old versions and advances 09 to 10 with collision lock',async()=>{const d=await mkdtemp(path.join(os.tmpdir(),'solara-release-test-'));try{const a=await reserveBuild(d,{minimum:9});assert.equal(a.publicVersion,'V-0.0.09');const b=await reserveBuild(d);assert.equal(b.publicVersion,'V-0.0.10');assert.equal(a.filename,'solaralauncher.exe');assert.equal(b.filename,a.filename);assert.notEqual(a.artifactPath,b.artifactPath);assert.equal(a.artifactPath,'V-0.0.09/solaralauncher.exe');await writeFile(path.join(d,'.allocation.lock'),'');await assert.rejects(()=>reserveBuild(d),/Another build/);const allocations=JSON.parse(await readFile(path.join(d,'allocations.json')));assert.equal(allocations.allocations.length,2);}finally{await rm(d,{recursive:true,force:true})}});
test('immutable artifacts cannot be replaced with changed bytes',async()=>{const d=await mkdtemp(path.join(os.tmpdir(),'solara-immutable-test-'));try{const src=path.join(d,'source'),target=path.join(d,'archive');await writeFile(src,'first');await immutableCopy(src,target);await immutableCopy(src,target);await writeFile(src,'different');await assert.rejects(()=>immutableCopy(src,target),/immutable/);assert.equal(await readFile(target,'utf8'),'first')}finally{await rm(d,{recursive:true,force:true})}});
test('minisign verifier rejects tampering and verifies both file and trusted metadata',()=>{const {publicKey,privateKey}=generateKeyPairSync('ed25519');const pk=publicKey.export({format:'der',type:'spki'}).subarray(-32);const id=Buffer.alloc(8,7),bytes=Buffer.from('actual executable test bytes');const signature=sign(null,createHash('blake2b512').update(bytes).digest(),privateKey);const comment='timestamp:123 file:Solara.exe';const blob=Buffer.concat([Buffer.from('ED'),id,signature]);const global=sign(null,Buffer.concat([signature,Buffer.from(comment)]),privateKey);const encoded=Buffer.from(`untrusted comment: test\n${blob.toString('base64')}\ntrusted comment: ${comment}\n${global.toString('base64')}\n`).toString('base64');const key=Buffer.from(`untrusted comment: key\n${Buffer.concat([Buffer.from('Ed'),id,pk]).toString('base64')}\n`).toString('base64');assert.equal(verifyMinisign(bytes,encoded,key),true);assert.throws(()=>verifyMinisign(Buffer.from('tampered'),encoded,key),/verification failed/);assert.throws(()=>verifyMinisign(bytes,encoded,'invalid'));assert.throws(()=>verifyPe(Buffer.from('fake exe')),/Windows executable/);});
