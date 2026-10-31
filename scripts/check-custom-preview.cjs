// Labelled native IPC fixtures: verifies preview limits, Fabric/Quilt loader picker, content install capability and device-code cancel UI.
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
   const state=window.__PREVIEW_QA__={calls:[],resolveLogin:null,failLoaders:true};
   window.__TAURI_INTERNALS__={transformCallback:fn=>{callbacks.set(++next,fn);return next},unregisterCallback:()=>{},invoke:async(command,args={})=>{
    state.calls.push({command,args});
    if(command==='plugin:event|listen'){events.set(args.event,args.handler);return events.size}
    if(command==='launcher_capabilities')return {loaders:['vanilla','fabric','quilt'],contentInstall:true,automaticJava:true,instanceFiles:true,modManager:true,preview:true};
    if(command==='launcher_settings')return {memory:4096,javaPath:'auto',discord:true};
    if(command==='launcher_snapshot')return structuredClone({instances:state.instances,downloads:[],minecraftAccounts:[],localServers:[]});
    if(command==='minecraft_versions')return [{id:'1.21.1',type:'release'}];
    if(command==='loader_versions'){
     if(state.failLoaders){state.failLoaders=false;throw Error('Temporary Fabric metadata failure')}
     if(args.loader==='quilt')return [{id:'0.30.1',stable:true},{id:'0.30.0-beta.1',stable:false}];
     return [{id:'0.20.1',stable:true},{id:'0.20.0+build.1',stable:false}];
    }
    if(command==='create_instance')return {jobId:'qa-job',instanceId:'qa-created'};
    if(command==='install_content')return null;
    if(command==='minecraft_sign_in'){
     callbacks.get(events.get('minecraft-login-code'))({event:'minecraft-login-code',id:1,payload:{code:'QA-CODE',url:'https://www.microsoft.com/link',expiresIn:900}});
     return new Promise(resolve=>{state.resolveLogin=resolve});
    }
    if(command==='cancel_minecraft_sign_in'){state.resolveLogin?.();return null}
    return null;
   }};
   state.instances=[{id:'qa-failed',name:'QA interrupted install',gameVersion:'1.21.1',loader:'vanilla',state:'failed',running:false}];
  });
  const page=await context.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto((process.env.SOLARA_UI_URL||'http://127.0.0.1:1420')+'/desktop.html',{waitUntil:'networkidle'});
  await page.locator('.boot-screen').waitFor({state:'hidden'});
  await page.getByRole('button',{name:'Minecraft launcher',exact:true}).click();
  await page.getByRole('button',{name:'Retry install',exact:true}).first().click();
  assert.equal(await page.evaluate(()=>window.__PREVIEW_QA__.calls.some(c=>c.command==='retry_instance'&&c.args.id==='qa-failed')),true);
  await page.getByRole('button',{name:'New instance',exact:true}).click();
  assert.equal(await page.getByRole('radio').count(),3);
  assert.equal(await page.getByRole('radio',{name:'Vanilla',exact:true}).count(),1);
  assert.equal(await page.getByRole('radio',{name:'Fabric',exact:true}).count(),1);
  assert.equal(await page.getByRole('radio',{name:'Quilt',exact:true}).count(),1);
  await page.getByRole('radio',{name:'Fabric',exact:true}).click();
  await page.getByText('Temporary Fabric metadata failure').waitFor();
  assert.equal(await page.getByRole('button',{name:'Create instance',exact:true}).isDisabled(),true);
  await page.getByRole('button',{name:'Try again',exact:true}).click();
  await page.getByRole('combobox',{name:'Loader version',exact:true}).click();
  await page.getByRole('option',{name:/0\.20\.1/}).click();
  await page.getByRole('radio',{name:'Quilt',exact:true}).click();
  await page.getByRole('combobox',{name:'Loader version',exact:true}).click();
  await page.getByRole('option',{name:/0\.30\.1/}).click();
  await page.getByPlaceholder('My next adventure',{exact:true}).fill('QA Quilt world');
  await page.getByRole('button',{name:'Create instance',exact:true}).click();
  await page.getByRole('dialog').waitFor({state:'hidden'});
  assert.deepEqual(await page.evaluate(()=>window.__PREVIEW_QA__.calls.find(c=>c.command==='create_instance')?.args),{name:'QA Quilt world',gameVersion:'1.21.1',loader:'quilt',loaderVersion:'0.30.1'});
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
  assert.equal(await page.getByRole('button',{name:'Install',exact:true}).isDisabled(),false);
  // With only a vanilla instance present, clicking Install on a mod warns that a Fabric/Quilt instance is required.
  await page.getByRole('button',{name:'Install',exact:true}).click();
  await page.getByText('Create a Fabric or Quilt instance before adding mods.',{exact:true}).waitFor();
  // Once a Quilt instance is present alongside the vanilla instance, only the Quilt instance is targeted.
  await page.evaluate(()=>{window.__PREVIEW_QA__.instances.push({id:'qa-quilt',name:'QA Quilt world',gameVersion:'1.21.1',loader:'quilt',state:'installed',running:false})});
  await page.getByRole('button',{name:'Minecraft launcher',exact:true}).click();
  await page.getByRole('heading',{name:'QA Quilt world',exact:true}).waitFor();
  await page.getByRole('button',{name:'Explore mods and modpacks',exact:true}).click();
  await page.getByRole('button',{name:'Search',exact:true}).click();
  await page.getByRole('button',{name:'Install',exact:true}).click();
  await page.getByRole('heading',{name:'Add to an instance',exact:true}).waitFor();
  assert.match(await page.getByRole('combobox',{name:'Install into',exact:true}).innerText(),/QA Quilt world/);
  await page.getByRole('button',{name:'Install content',exact:true}).click();
  await page.getByRole('dialog').waitFor({state:'hidden'});
  assert.deepEqual(await page.evaluate(()=>window.__PREVIEW_QA__.calls.find(c=>c.command==='install_content')?.args),{instanceId:'qa-quilt',projectId:'qa-mod',contentType:'mod',versionId:null});
  assert.deepEqual(errors,[]);
  console.log('Custom preview capability, Fabric/Quilt loader picker, mod-capable instance filtering, interrupted-install retry and device-code cancellation fixtures passed. No real account/game used.');
 }finally{await browser.close()}
})().catch(e=>{console.error(e);process.exitCode=1});
