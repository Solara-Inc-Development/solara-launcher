import {test} from 'node:test';import assert from 'node:assert/strict';
import {chooseEvent,NEUTRAL,publicVersion,buildNumber,latestRelease,isDownloadable,PublicCache} from '../packages/shared/domain.mjs';
import {readFileSync} from 'node:fs';
const {events}=JSON.parse(readFileSync(new URL('../packages/shared/events.json',import.meta.url)));
const data={schemaVersion:1,events,projects:[],services:[],releases:[]};
test('catalogue preserves existing entries and configures every required occasion',()=>{
 assert.ok(events.length>=50);for(const id of ['ugadi','early_winter','guru_gobind_singh','gandhi_jayanti','chaitra_navratri','navratri','vasant_panchami','diwali'])assert.ok(events.some(e=>e.id===id));
 for(const e of events){assert.equal(e.timezone,'Asia/Kolkata');assert.match(e.accent,/^#[a-f0-9]{6}$/);assert.ok(e.assets);assert.ok(e.banner.copy);for(const o of e.occurrences)assert.ok(Date.parse(o.start)<Date.parse(o.end));}
 assert.equal(events.find(e=>e.id==='vasant_panchami').occurrences.length,0);
});
test('India UTC boundary, exclusive end, neutral choice, previews and expired schedules',()=>{
 const a=chooseEvent(events,{now:Date.parse('2026-01-25T18:29:59Z')});assert.notEqual(a.id,'republic_day');
 assert.equal(chooseEvent(events,{now:Date.parse('2026-01-25T18:30:00Z')}).id,'republic_day');
 assert.notEqual(chooseEvent(events,{now:Date.parse('2026-01-26T18:30:00Z')}).id,'republic_day');
 assert.equal(chooseEvent(events,{mode:'neutral',preview:'diwali'}),NEUTRAL);
 assert.equal(chooseEvent(events,{preview:'diwali'}).id,'diwali');
 assert.equal(chooseEvent(events,{now:Date.parse('2040-01-01')}),NEUTRAL);
 assert.equal(chooseEvent(events,{now:Date.parse('2026-11-08T06:00Z')}).id,'diwali');
 assert.equal(chooseEvent(events,{override:{eventId:'holi',expiresAt:'2000-01-01'}}).id,chooseEvent(events).id);
});
test('overlaps are deterministic; disabled and regional exclusions work',()=>{
 const f=id=>({id,name:id,enabled:true,priority:100,accent:'#123456',regions:['IN'],occurrences:[{start:'2026-01-01',end:'2027-01-01',verified:true}]});
 assert.equal(chooseEvent([f('z'),f('a')],{now:Date.parse('2026-07-01')}).id,'a');assert.equal(chooseEvent([f('a')],{region:'US'}),NEUTRAL);
 assert.equal(chooseEvent([{...f('a'),enabled:false}]),NEUTRAL);
});
test('versions increment and compare numerically across 09 to 10',()=>{assert.equal(publicVersion(9),'V-0.0.09');assert.equal(publicVersion(10),'V-0.0.10');assert.equal(buildNumber('V-0.0.100'),100);assert.throws(()=>publicVersion(0));});
const release=n=>({state:'published',platform:'windows',arch:'x86_64',verifiedAt:'2026-10-28T00:00Z',signature:'test-signature',size:123456,sha256:'a'.repeat(64),publicVersion:publicVersion(n),internalVersion:`0.0.${n}`,filename:`Solara-Launcher-${publicVersion(n)}.exe`,url:`https://launcher.solara-inc.in/builds/Solara-Launcher-${publicVersion(n)}.exe`});
test('downloads require real verified metadata and hide withdrawn or incomplete releases',()=>{assert.equal(latestRelease([release(9),release(10)]).publicVersion,'V-0.0.10');for(const r of [{...release(1),state:'withdrawn'},{...release(1),signature:null},{...release(1),sha256:'x'},{...release(1),url:'javascript:alert(1)'},{...release(1),filename:'fake.exe'}])assert.equal(isDownloadable(r),false);});
test('cache handles outages, corruption, expiry, rate limits and never sends identity',async()=>{
 let now=100000;const storage={value:null,getItem(){return this.value},setItem(k,v){this.value=v}};let requests=0;
 const c=new PublicCache({storage,clock:()=>now,fetcher:async(url,init)=>{requests++;assert.equal(init.credentials,'omit');assert.equal(init.headers.Authorization,undefined);return new Response(JSON.stringify(data))}});
 assert.equal((await c.refresh()).state,'connected');c.fetcher=async()=>{throw Error('offline')};assert.equal((await c.refresh()).state,'cached');
 now+=20000;c.fetcher=async()=>new Response('',{status:429,headers:{'Retry-After':'9999999'}});const r=await c.refresh();assert.equal(r.state,'rate-limited');assert.equal(r.retryAt-now,300000);assert.equal((await c.refresh()).state,'rate-limited');
 now+=8*86400000;assert.equal(c.read(),null);storage.value='bad';assert.equal(c.read(),null);assert.equal(requests,1);
});
