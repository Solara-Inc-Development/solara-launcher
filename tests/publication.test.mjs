import {test} from 'node:test';
import assert from 'node:assert/strict';
import {generateKeyPairSync,sign,createHash} from 'node:crypto';
import {validateRelease,verifyReleaseAssets} from '../scripts/prepare-github-release.mjs';

// Header/signature fixtures live only in memory, never distributable artifacts.
function fixture(){
 const exe=Buffer.alloc(1025);exe.write('MZ');exe.writeUInt32LE(128,0x3c);exe.write('PE\0\0',128);exe.writeUInt16LE(0x14c,132);exe.writeUInt16LE(1,134);exe.writeUInt16LE(240,148);exe.writeUInt16LE(2,150);exe.writeUInt16LE(0x10b,152);
 const source=Buffer.alloc(128);source.writeUInt32LE(0x04034b50,0);
 const {publicKey,privateKey}=generateKeyPairSync('ed25519'),id=Buffer.alloc(8,4);
 const sig=sign(null,createHash('blake2b512').update(exe).digest(),privateKey),comment='timestamp:123\tversion:0.0.14';
 const global=sign(null,Buffer.concat([sig,Buffer.from(comment)]),privateKey);
 const signature=Buffer.from(`untrusted comment: QA\n${Buffer.concat([Buffer.from('ED'),id,sig]).toString('base64')}\ntrusted comment: ${comment}\n${global.toString('base64')}\n`).toString('base64');
 const key=Buffer.from(`untrusted comment: QA\n${Buffer.concat([Buffer.from('Ed'),id,publicKey.export({format:'der',type:'spki'}).subarray(-32)]).toString('base64')}\n`).toString('base64');
 const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
 const release={publicVersion:'V-0.0.14',internalVersion:'0.0.14',filename:'solaralauncher.exe',artifactPath:'V-0.0.14/solaralauncher.exe',platform:'windows',arch:'x86_64',state:'published',verifiedAt:new Date().toISOString(),url:'https://app.solara-inc.in/builds/V-0.0.14/solaralauncher.exe',size:exe.length,sha256:sha(exe),signature,artifacts:{'source.zip':{size:source.length,sha256:sha(source)}}};
 return {release,key,assets:{'solaralauncher.exe':exe,'solaralauncher.exe.sig':Buffer.from(signature),'source.zip':source}};
}
test('release publication requires canonical version and exact-source metadata',()=>{
 const {release}=fixture();assert.match(validateRelease(release,'V-0.0.14'),/V-0.0.14/);
 assert.throws(()=>validateRelease(release,'V-0.0.15'),/published/);
 assert.throws(()=>validateRelease({...release,state:'staged'},'V-0.0.14'),/published/);
 assert.throws(()=>validateRelease({...release,artifacts:{}},'V-0.0.14'),/source/);
 assert.throws(()=>validateRelease({...release,url:'https://example.com/V-0.0.14/solaralauncher.exe'},'V-0.0.14'),/official/);
});
test('release publication rejects changed installer, signature or corresponding source',()=>{
 const {release,key,assets}=fixture();verifyReleaseAssets(release,'V-0.0.14',assets,key);
 for(const name of ['solaralauncher.exe','source.zip']){
  const changed=Buffer.from(assets[name]);changed[changed.length-1]^=1;
  assert.throws(()=>verifyReleaseAssets(release,'V-0.0.14',{...assets,[name]:changed},key),/match/);
 }
 assert.throws(()=>verifyReleaseAssets(release,'V-0.0.14',{...assets,'solaralauncher.exe.sig':Buffer.from('wrong signature')},key),/Signature/);
});
