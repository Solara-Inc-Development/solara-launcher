// Labelled native IPC fixtures: verifies preview limits and device-code cancel UI.
const {chromium}=require('playwright');
const assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true,args:['--no-sandbox']});
 try{
  const context=await browser.newContext({viewport:{width:1440,height:950}});
  await context.route('**/v1/launcher/bootstrap',r=>r.fulfill({status:503,body:'QA outage'}));
  await context.route('https://api.modrinth.com/v2/search?**',r=>r.fulfill({json:{hits:[{project_id:'qa-mod',project_type:'mod',title:'QA preview mod',description:'Fixture only',author:'QA'}]}}));
  await context.addInitScript(()=>{
   const callbacks=new Map(),events=new Map();let next=0;
   const state=window.__PREVIEW_QA__={calls:[],resolveLogin:null};
   window.__TAURI_INTERNALS__={transformCallback:fn=>{callbacks.set(++next,fn);return next},unregisterCallback:()=>{},invoke:async(command,args={})=>{
    state.calls.push({command,args});
    if(command==='plugin:event|listen'){events.set(args.event,args.handler);return events.size}
    if(command==='launcher_capabilities')return {loaders:['vanilla'],contentInstall:false,preview:true};
    if(command==='launcher_settings')return {memory:4096,javaPath:'java',discord:false};
    if(command==='launcher_snapshot')return {instances:[{id:'qa-failed',name:'QA interrupted install',gameVersion:'1.21.1',loader:'vanilla',state:'failed',running:false}],downloads:[],minecraftAccounts:[],localServers:[]};
    if(command==='minecraft_versions')return [{id:'1.21.1',type:'release'}];
    if(command==='minecraft_sign_in'){
     callbacks.get(events.get('minecraft-login-code'))({event:'minecraft-login-code',id:1,payload:{code:'QA-CODE',url:'https://www.microsoft.com/link',expiresIn:900}});
     return new Promise(resolve=>{state.resolveLogin=resolve});
    }
    if(command==='cancel_minecraft_sign_in'){state.resolveLogin?.();return null}
    return null;
   }};
  });
  const page=await context.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto((process.env.SOLARA_UI_URL||'http://127.0.0.1:1420')+'/desktop.html',{waitUntil:'networkidle'});
  await page.locator('.boot-screen').waitFor({state:'hidden'});
  await page.getByRole('button',{name:'Minecraft launcher',exact:true}).click();
  await page.getByRole('button',{name:'Retry install',exact:true}).first().click();
  assert.equal(await page.evaluate(()=>window.__PREVIEW_QA__.calls.some(c=>c.command==='retry_instance'&&c.args.id==='qa-failed')),true);
  await page.getByRole('button',{name:'New instance',exact:true}).click();
  assert.equal(await page.getByRole('radio').count(),1);
  assert.equal(await page.getByRole('radio',{name:'Vanilla',exact:true}).count(),1);
  await page.getByRole('button',{name:'Close dialog',exact:true}).click();
  // Locate the primary account action by its existing exact label.
  await page.getByRole('button',{name:'Add account',exact:true}).click();
  await page.getByRole('heading',{name:'Sign in to Minecraft',exact:true}).waitFor();
  assert.equal(await page.locator('.minecraft-device-code').innerText(),'QA-CODE');
  await page.getByRole('button',{name:'Cancel sign-in',exact:true}).click();
  await page.getByRole('dialog').waitFor({state:'hidden'});
  assert.equal(await page.evaluate(()=>window.__PREVIEW_QA__.calls.some(c=>c.command==='cancel_minecraft_sign_in')),true);
  await page.getByRole('button',{name:'Explore mods and modpacks',exact:true}).click();
  await page.getByRole('button',{name:'Search',exact:true}).click();
  await page.getByRole('heading',{name:'QA preview mod',exact:true}).waitFor();
  assert.equal(await page.getByRole('button',{name:'Install',exact:true}).isDisabled(),true);
  assert.deepEqual(errors,[]);
  console.log('Custom preview capability, interrupted-install retry and device-code cancellation fixtures passed. No real account/game used.');
 }finally{await browser.close()}
})().catch(e=>{console.error(e);process.exitCode=1});
