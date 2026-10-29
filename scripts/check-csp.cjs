// Browser policy check using the actual desktop CSP; no game/provider credentials.
const {chromium}=require('playwright');
const {readFileSync}=require('node:fs');
const assert=require('node:assert/strict');

(async()=>{
 const {providerIcon}=await import('../packages/shared/content.mjs');
 const csp=JSON.parse(readFileSync('apps/desktop/tauri.conf.json','utf8')).app.security.csp;
 const browser=await chromium.launch({headless:true,args:process.env.CI?[]:['--no-sandbox']});
 try{
  const context=await browser.newContext();
  const origin='http://solara.test',allowed=providerIcon('https://cdn.modrinth.com/data/qa-fixture/icon.svg');
  const disallowed='https://untrusted.invalid/icon.svg';
  let allowedRequests=0,disallowedRequests=0;
  const svg='<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="green"/></svg>';
  await context.route(origin+'/',r=>r.fulfill({contentType:'text/html',headers:{'content-security-policy':csp},body:`<!doctype html><img id="local" src="/logo.svg"><img id="provider" src="${allowed}" referrerpolicy="no-referrer"><img id="blocked" src="${disallowed}">`}));
  await context.route(origin+'/logo.svg',r=>r.fulfill({contentType:'image/svg+xml',body:svg}));
  await context.route(allowed,r=>{allowedRequests++;assert.equal(r.request().headers().referer,undefined);return r.fulfill({contentType:'image/svg+xml',body:svg})});
  await context.route(disallowed,r=>{disallowedRequests++;return r.fulfill({contentType:'image/svg+xml',body:svg})});
  const page=await context.newPage();
  await page.goto(origin+'/',{waitUntil:'networkidle'});
  assert.equal(await page.locator('#local').evaluate(img=>img.naturalWidth),16);
  assert.equal(await page.locator('#provider').evaluate(img=>img.naturalWidth),16,'Validated provider icons must load under the actual desktop CSP');
  assert.equal(allowedRequests,1);
  assert.equal(await page.locator('#blocked').evaluate(img=>img.naturalWidth),0);
  assert.equal(disallowedRequests,0,'Untrusted image origin must be blocked before a network request');
  console.log('PASS: desktop CSP permits local/trusted Modrinth icons, omits referrer and blocks untrusted images (browser fixtures).');
 }finally{await browser.close()}
})().catch(error=>{console.error(error);process.exit(1)});
