export const API_ORIGIN = 'https://api.solara-inc.in';
export const NEUTRAL = Object.freeze({id:'none',name:'Classic',accent:'#3dd6f5',background:'#0b0e12',effects:[],banner:{title:'Make room for your next world.',copy:'Your games. Your services. One home.'}});
export function safeHttps(value) {
  try { const u=new URL(value); return u.protocol==='https:' && !u.username && !u.password ? u.href : null; } catch { return null; }
}
/** @param {any[]} events @param {any} options */
export function chooseEvent(events, {now=Date.now(), region='IN', mode='auto', preview=null, override=null}={}) {
  if(mode==='neutral') return NEUTRAL;
  const valid=(events||[]).filter(e=>e && e.enabled===true && typeof e.id==='string' && /^#[0-9a-f]{6}$/i.test(e.accent) && Number.isFinite(e.priority));
  if(preview) return valid.find(e=>e.id===preview) || NEUTRAL;
  if(override && Date.parse(override.expiresAt)>now) return valid.find(e=>e.id===override.eventId) || NEUTRAL;
  return valid.filter(e=>(e.regions.includes(region)||e.regions.includes('global')) && e.occurrences.some(o=>o.verified===true && Date.parse(o.start)<=now && now<Date.parse(o.end)))
    .sort((a,b)=>b.priority-a.priority || a.id.localeCompare(b.id))[0] || NEUTRAL;
}
export function publicVersion(n) {
  if(!Number.isSafeInteger(n)||n<1) throw new Error('Build number must be a positive integer');
  return `V-0.0.${String(n).padStart(2,'0')}`;
}
export function buildNumber(version) {
  const m=/^V-0\.0\.(\d+)$/.exec(version||'');
  if(!m||!Number.isSafeInteger(Number(m[1]))||Number(m[1])<1) throw new Error('Invalid public version');
  return Number(m[1]);
}
export function releaseArtifactPath(r) {
  const version=publicVersion(buildNumber(r.publicVersion));
  if(r.publicVersion!==version)throw new Error('Noncanonical version');
  const expected=r.filename==='solaralauncher.exe'?`${version}/solaralauncher.exe`:`Solara-Launcher-${version}.exe`;
  if(r.filename!=='solaralauncher.exe' && r.filename!==expected)throw new Error('Invalid release filename');
  if(r.artifactPath!==undefined && r.artifactPath!==expected)throw new Error('Invalid artifact path');
  return expected;
}
export function isDownloadable(r) {
  if(!r || r.state!=='published'||r.platform!=='windows'||r.arch!=='x86_64'||!r.verifiedAt||!Number.isFinite(Date.parse(r.verifiedAt))||!r.signature || !/^[a-f0-9]{64}$/i.test(r.sha256||'') || !Number.isSafeInteger(r.size)||r.size<1024 || !safeHttps(r.url)) return false;
  try {const n=buildNumber(r.publicVersion);return r.internalVersion===`0.0.${n}` && new URL(r.url).pathname.endsWith('/'+releaseArtifactPath(r));}catch{return false;}
}
export function latestRelease(releases) {
  return (releases||[]).filter(isDownloadable).sort((a,b)=>buildNumber(b.publicVersion)-buildNumber(a.publicVersion))[0] || null;
}
export function validBootstrap(d) {
  return !!d && d.schemaVersion===1 && Array.isArray(d.events) && d.events.length<=500 && d.events.every(e=>e && typeof e.id==='string' && e.id.length<=80 && typeof e.name==='string' && Array.isArray(e.regions) && Array.isArray(e.occurrences) && e.occurrences.length<=100) && Array.isArray(d.projects) && Array.isArray(d.services) && [...d.projects,...d.services].length<=500 && [...d.projects,...d.services].every(x=>typeof x.id==='string' && typeof x.name==='string' && (!x.url || !!safeHttps(x.url))) && Array.isArray(d.releases);
}
export class PublicCache {
  /** @param {any} options */
  constructor({storage,key='solara:bootstrap:v1',fetcher=globalThis.fetch,clock=Date.now,origin=API_ORIGIN}={}) {
    if(origin!==API_ORIGIN && !/^http:\/\/(127\.0\.0\.1|localhost):\d+$/.test(origin)) throw new Error('Only the main Solara API or explicit local development is supported');
    this.storage=storage;this.key=key;this.fetcher=fetcher.bind(globalThis);this.clock=clock;this.origin=origin;this.retryAt=0;
  }
  read() {
    try {const c=JSON.parse(this.storage?.getItem(this.key)||'null');return c && validBootstrap(c.data)&&Number.isFinite(c.at)&&this.clock()-c.at<7*86400000&&c.at<=this.clock()?c:null;}catch{return null;}
  }
  async refresh() {
    const cached=this.read();
    if(this.clock()<this.retryAt) return {data:cached?.data||null,at:cached?.at||null,state:'rate-limited',retryAt:this.retryAt};
    try {
      const response=await this.fetcher(`${this.origin}/v1/launcher/bootstrap`,{credentials:'omit',headers:{Accept:'application/json'},signal:AbortSignal.timeout(8000)});
      if(response.status===429) {
        const raw=response.headers.get('Retry-After');const seconds=Number(raw);const delay=raw && Number.isFinite(seconds)?seconds*1000:Date.parse(raw||'')-this.clock();
        this.retryAt=this.clock()+Math.min(300000,Math.max(1000,Number.isFinite(delay)?delay:30000));
        return {data:cached?.data||null,at:cached?.at||null,state:'rate-limited',retryAt:this.retryAt};
      }
      if(!response.ok) throw new Error('API unavailable');
      const content=await response.text();if(content.length>2_000_000) throw new Error('Response too large');
      const data=JSON.parse(content);if(!validBootstrap(data)) throw new Error('Invalid bootstrap');
      const at=this.clock();try {this.storage?.setItem(this.key,JSON.stringify({data,at}));}catch{}
      return {data,at,state:'connected'};
    }catch{this.retryAt=this.clock()+15000;return {data:cached?.data||null,at:cached?.at||null,state:cached?'cached':'unavailable'};}
  }
}
