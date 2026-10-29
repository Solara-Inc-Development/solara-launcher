// Explicit browser IPC fixtures; filesystem safety is covered by native tests.
const {chromium}=require('playwright');const assert=require('node:assert/strict');
(async()=>{const browser=await chromium.launch({headless:true,args:['--no-sandbox']});try{
 const context=await browser.newContext({viewport:{width:1440,height:1000}});
 await context.route('**/v1/launcher/bootstrap',r=>r.fulfill({status:503,body:'QA outage'}));
 await context.addInitScript(()=>{
  const id='12345678-1234-4234-8234-123456789abc';
  const state=window.__WORKSPACE_QA__={calls:[],instance:{id,name:'QA test world',gameVersion:'1.21.1',loader:'fabric',loaderVersion:'0.20.1',state:'installed',running:false,memory:4096,javaPath:'auto'},files:{'options.txt':{content:'fixture-original',sha256:'fixture-v1'}},trash:[],modEnabled:true};
  window.__TAURI_INTERNALS__={transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(command,args={})=>{
   state.calls.push({command,args});
   if(command==='plugin:event|listen')return 1;
   if(command==='launcher_capabilities')return {loaders:['vanilla','fabric'],contentInstall:false,automaticJava:true,instanceFiles:true,modManager:true,preview:true};
   if(command==='launcher_settings')return {memory:4096,javaPath:'auto',discord:false};
   if(command==='launcher_snapshot')return structuredClone({instances:[state.instance],minecraftAccounts:[{id:'qa-account',name:'QA player',active:true}],localServers:[],downloads:[]});
   if(command==='instance_files')return Object.keys(state.files).map(path=>({name:path,path,directory:false,size:state.files[path].content.length}));
   if(command==='instance_text')return {path:args.path,...state.files[args.path]};
   if(command==='instance_file_action'){
    if(args.action==='save'){const file=state.files[args.path];if(file&&args.expectedSha256!==file.sha256)throw Error('This file changed since you opened it. Reload it before saving.');state.files[args.path]={content:args.content,sha256:'fixture-v'+Date.now()};return {sha256:state.files[args.path].sha256}}
    if(args.action==='rename'){state.files[args.target]=state.files[args.path];delete state.files[args.path];return {ok:true}}
    if(args.action==='trash'){state.trash.push({id:'qa-trash',originalPath:args.path,removedAt:'2026-10-29T12:00:00Z',directory:false,payload:state.files[args.path]});delete state.files[args.path];return {trashId:'qa-trash'}}
    if(args.action==='restore'){const item=state.trash.find(t=>t.id===args.path);state.files[item.originalPath]=item.payload;state.trash=[];return {ok:true}}
   }
   if(command==='instance_trash')return structuredClone(state.trash);
   if(command==='instance_mods')return [{path:'mods/fixture.jar'+(state.modEnabled?'':'.disabled'),fileName:'fixture.jar',name:'QA mod',version:'1.2.3',loaders:['fabric'],enabled:state.modEnabled,size:128}];
   if(command==='toggle_instance_mod'){state.modEnabled=args.enabled;return null}
   if(command==='instance_logs')return 'Explicit QA log fixture';
   if(command==='edit_instance'){Object.assign(state.instance,args);return null}
   return null;
  }};
 });
 const page=await context.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto((process.env.SOLARA_UI_URL||'http://127.0.0.1:1420')+'/desktop.html',{waitUntil:'networkidle'});await page.locator('.boot-screen').waitFor({state:'hidden'});
 await page.getByRole('button',{name:'Minecraft launcher',exact:true}).click();await page.getByRole('button',{name:'Open instance QA test world',exact:true}).click();
 await page.getByRole('navigation',{name:'Instance pages'}).getByRole('button',{name:'Files',exact:true}).click();
 await page.getByRole('button',{name:'options.txt',exact:true}).click();await page.getByRole('textbox',{name:'File contents'}).fill('fixture-saved');await page.locator('.workspace-editor-bar').getByRole('button',{name:'Save',exact:true}).click();
 assert.equal(await page.evaluate(()=>window.__WORKSPACE_QA__.files['options.txt'].content),'fixture-saved');
 await page.getByRole('textbox',{name:'File contents'}).fill('unsaved-fixture');
 await page.locator('.sidebar').getByRole('button',{name:'Settings',exact:true}).click();await page.getByRole('dialog').getByRole('button',{name:'Cancel',exact:true}).click();assert.equal(await page.getByRole('textbox',{name:'File contents'}).inputValue(),'unsaved-fixture');
 await page.getByRole('button',{name:'Close file editor',exact:true}).click();await page.getByRole('dialog').getByRole('button',{name:'Discard changes',exact:true}).click();
 await page.getByRole('button',{name:'Rename options.txt',exact:true}).click();await page.getByRole('dialog').getByLabel('Name',{exact:true}).fill('renamed.txt');await page.getByRole('dialog').getByRole('button',{name:'Save',exact:true}).click();
 await page.getByRole('button',{name:'Move renamed.txt to Trash',exact:true}).click();assert.equal(await page.evaluate(()=>Object.keys(window.__WORKSPACE_QA__.files).length),1);await page.getByRole('dialog').getByRole('button',{name:'Move to Trash',exact:true}).click();
 await page.getByRole('navigation',{name:'Instance pages'}).getByRole('button',{name:'Trash',exact:true}).click();await page.getByRole('button',{name:'Restore',exact:true}).click();assert.equal(await page.evaluate(()=>window.__WORKSPACE_QA__.files['renamed.txt'].content),'fixture-saved');
 await page.getByRole('navigation',{name:'Instance pages'}).getByRole('button',{name:'Mods',exact:true}).click();await page.getByRole('switch',{name:'Disable QA mod',exact:true}).click();assert.equal(await page.getByRole('switch',{name:'Enable QA mod',exact:true}).getAttribute('aria-checked'),'false');
 await page.evaluate(()=>window.__WORKSPACE_QA__.instance.running=true);await page.getByRole('button',{name:'Stop game',exact:true}).waitFor();assert.equal(await page.getByRole('switch',{name:'Enable QA mod',exact:true}).isDisabled(),true);
 await page.setViewportSize({width:800,height:700});assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);await page.screenshot({path:'checks/instance-workspace-fixture.png',animations:'disabled'});
 assert.deepEqual(errors,[]);console.log('Instance workspace edit/save, unsaved navigation, rename/trash/restore, mod toggle and running locks PASS. Explicit IPC fixtures only.');
}finally{await browser.close()}})().catch(e=>{console.error(e);process.exitCode=1});
