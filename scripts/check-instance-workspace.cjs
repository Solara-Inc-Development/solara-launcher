// Explicit browser IPC fixtures; filesystem safety is covered by native tests.
const {chromium}=require('playwright');const assert=require('node:assert/strict');
(async()=>{const browser=await chromium.launch({headless:true,args:['--no-sandbox']});try{
 const context=await browser.newContext({viewport:{width:1440,height:1000}});
 await context.route('**/v1/launcher/bootstrap',r=>r.fulfill({status:503,body:'QA outage'}));
 await context.addInitScript(()=>{
  const id='12345678-1234-4234-8234-123456789abc';
  const state=window.__WORKSPACE_QA__={calls:[],instance:{id,name:'QA test world',gameVersion:'1.21.1',loader:'fabric',loaderVersion:'0.20.1',state:'installed',running:false,memory:4096,javaPath:'auto'},files:{'options.txt':{content:'fixture-original',sha256:'fixture-v1'}},trash:[],backups:[],importedMods:[],modEnabled:true,downloads:[]};
  window.__TAURI_INTERNALS__={transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(command,args={})=>{
   state.calls.push({command,args});
   if(command==='plugin:event|listen')return 1;
   if(command==='launcher_capabilities')return {loaders:['vanilla','fabric','quilt'],contentInstall:true,automaticJava:true,instanceFiles:true,modManager:true,instanceBackups:true,preview:true};
   if(command==='launcher_settings')return {memory:4096,javaPath:'auto',discord:true};
   if(command==='launcher_snapshot')return structuredClone({instances:[state.instance],minecraftAccounts:[{id:'qa-account',name:'QA player',active:true}],localServers:[],downloads:state.downloads||[]});
   if(command==='clear_finished_downloads'){state.downloads=(state.downloads||[]).filter(j=>['running','queued','downloading'].includes(j.status));return null;}
   if(command==='instance_files'){
     if(args.path==='screenshots')return [{name:'2026-11-06_18.20.15.png',path:'screenshots/2026-11-06_18.20.15.png',directory:false,size:1024,modifiedAt:1730900000}];
     return Object.keys(state.files).map(path=>({name:path,path,directory:false,size:state.files[path].content.length}));
   }
   if(command==='instance_image')return 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';
   if(command==='instance_text')return {path:args.path,...state.files[args.path]};
   if(command==='instance_file_action'){
    if(args.action==='save'){const file=state.files[args.path];if(file&&args.expectedSha256!==file.sha256)throw Error('This file changed since you opened it. Reload it before saving.');state.files[args.path]={content:args.content,sha256:'fixture-v'+Date.now()};return {sha256:state.files[args.path].sha256}}
    if(args.action==='rename'){state.files[args.target]=state.files[args.path];delete state.files[args.path];return {ok:true}}
    if(args.action==='trash'){state.trash.push({id:'qa-trash',originalPath:args.path,removedAt:'2026-10-29T12:00:00Z',directory:false,payload:state.files[args.path]});delete state.files[args.path];return {trashId:'qa-trash'}}
    if(args.action==='restore'){const item=state.trash.find(t=>t.id===args.path);state.files[item.originalPath]=item.payload;state.trash=[];return {ok:true}}
   }
   if(command==='instance_trash')return structuredClone(state.trash);
   if(command==='instance_backups')return structuredClone(state.backups.map(({snapshot,...meta})=>meta));
   if(command==='instance_backup_action'){
    if(args.action==='create'){const entry={id:'qa-backup-1',name:args.name,createdAt:'2026-10-29T12:05:00Z',size:256,fileCount:Object.keys(state.files).length,worldCount:1,sha256:'a'.repeat(64),snapshot:structuredClone(state.files)};state.backups.push(entry);return entry}
    if(args.action==='restore'){const entry=state.backups.find(b=>b.id===args.backupId);state.files=structuredClone(entry.snapshot);return entry}
    if(args.action==='delete'){state.backups=state.backups.filter(b=>b.id!==args.backupId);return {ok:true}}
   }
   if(command==='instance_mods')return [{path:'mods/fixture.jar'+(state.modEnabled?'':'.disabled'),fileName:'fixture.jar',name:'QA mod',modId:'qa_mod',version:'1.2.3',description:'Essential performance and rendering optimization mod',authors:['QA Author'],loaders:['fabric'],enabled:state.modEnabled,size:128},...state.importedMods];
   if(command==='toggle_instance_mod'){state.modEnabled=args.enabled;return null}
   if(command==='import_instance_mod'){const mod={path:`mods/${args.fileName}`,fileName:args.fileName,name:'Imported QA mod',version:'2.0.0',loaders:['fabric'],enabled:true,size:256};state.importedMods.push(mod);return mod}
   if(command==='instance_logs')return 'Explicit QA log fixture';
   if(command==='edit_instance'){Object.assign(state.instance,args);return null}
   if(command==='open_instance_folder'||command==='open_instance_file')return null;
   if(command==='duplicate_instance'){const copy={...state.instance,id:'qa-duplicate-id',name:args.name||`${state.instance.name} (Copy)`};state.duplicatedInstance=copy;return copy}
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
 // Exercise isolated world/config backups (create, restore, delete)
 await page.getByRole('navigation',{name:'Instance pages'}).getByRole('button',{name:'Backups',exact:true}).click();
 await page.getByRole('button',{name:'Create backup',exact:true}).first().click();
 await page.getByRole('dialog').getByLabel('Name',{exact:true}).fill('Before boss');
 await page.getByRole('dialog').getByRole('button',{name:'Create backup',exact:true}).click();
 await page.getByRole('dialog').waitFor({state:'hidden'});
 await page.getByText('Before boss',{exact:true}).waitFor();
 await page.evaluate(()=>{window.__WORKSPACE_QA__.files['renamed.txt'].content='corrupted-after-backup'});
 await page.getByRole('button',{name:'Restore backup Before boss',exact:true}).click();
 await page.getByRole('dialog').getByRole('button',{name:'Restore backup',exact:true}).click();
 await page.getByRole('dialog').waitFor({state:'hidden'});
 assert.equal(await page.evaluate(()=>window.__WORKSPACE_QA__.files['renamed.txt'].content),'fixture-saved');
 await page.getByRole('button',{name:'Delete backup Before boss',exact:true}).click();
 await page.getByRole('dialog').getByRole('button',{name:'Delete backup',exact:true}).click();
 await page.getByRole('dialog').waitFor({state:'hidden'});
 assert.equal(await page.evaluate(()=>window.__WORKSPACE_QA__.backups.length),0);
 // Exercise local .jar mod import and mod toggle
 await page.getByRole('navigation',{name:'Instance pages'}).getByRole('button',{name:/^Mods/}).click();
 await page.locator('input[type="file"]').setInputFiles({name:'custom-zoom.jar',mimeType:'application/java-archive',buffer:Buffer.from('PK\x03\x04fixture-jar')});
 await page.getByText('Imported QA mod',{exact:true}).waitFor();
 assert.equal(await page.evaluate(()=>window.__WORKSPACE_QA__.calls.some(c=>c.command==='import_instance_mod'&&c.args.fileName==='custom-zoom.jar')),true);
 await page.getByRole('switch',{name:'Disable QA mod',exact:true}).click();assert.equal(await page.getByRole('switch',{name:'Enable QA mod',exact:true}).getAttribute('aria-checked'),'false');

 // Exercise Mod Inspector Dialog
 await page.getByRole('button',{name:'Inspect QA mod',exact:true}).click();
 const modInspector=page.locator('.mod-inspector-dialog');
 await modInspector.waitFor({state:'visible'});
 await modInspector.getByRole('heading',{name:'QA mod',exact:true}).waitFor();
 await modInspector.getByText('v1.2.3',{exact:true}).waitFor();
 await modInspector.getByText('QA Author',{exact:true}).waitFor();
 await modInspector.getByText('qa_mod',{exact:true}).waitFor();
 // Exercise 1-click toggle inside inspector (enable and disable)
 await modInspector.getByRole('button',{name:'Enable Mod',exact:true}).click();
 await modInspector.getByRole('button',{name:'Disable Mod',exact:true}).waitFor();
 await modInspector.getByRole('button',{name:'Disable Mod',exact:true}).click();
 await modInspector.getByRole('button',{name:'Enable Mod',exact:true}).waitFor();
 // Exercise 1-click open in explorer action
 await modInspector.getByRole('button',{name:'Open in Explorer',exact:true}).click();
 assert.equal(await page.evaluate(()=>window.__WORKSPACE_QA__.calls.some(c=>c.command==='open_instance_folder'&&c.args.path==='mods')),true);
 // Exercise 1-click delete mod action with confirmation
 await modInspector.getByRole('button',{name:'Delete mod',exact:true}).click();
 await modInspector.getByText('Move QA mod to Trash?',{exact:true}).waitFor();
 await modInspector.getByRole('button',{name:'Cancel',exact:true}).click();
 await modInspector.getByRole('button',{name:'Done',exact:true}).click();
 await modInspector.waitFor({state:'hidden'});

 // Exercise smart archive drop routing for multi-file .zip archives
 await page.locator('input[type="file"]').setInputFiles([
   {name:'faithful32.zip',mimeType:'application/zip',buffer:Buffer.from('PK\x03\x04fixture-zip1')},
   {name:'barebones.zip',mimeType:'application/zip',buffer:Buffer.from('PK\x03\x04fixture-zip2')}
 ]);
 await page.getByRole('dialog').getByText('Archive File Detected',{exact:true}).waitFor();
 await page.getByRole('dialog').getByText('2 archive files (faithful32.zip + 1 more)',{exact:true}).waitFor();
 await page.getByRole('dialog').getByText('Open Resource Packs Folder',{exact:true}).waitFor();
 await page.getByRole('dialog').getByRole('button',{name:'Close',exact:true}).click();
 await page.getByRole('dialog').waitFor({state:'hidden'});

 // Exercise screenshot gallery and lightbox with zoom, copy & trash trigger
 await page.getByRole('navigation',{name:'Instance pages'}).getByRole('button',{name:'Screenshots',exact:true}).click();
 await page.getByRole('button',{name:'View screenshot 2026-11-06_18.20.15.png',exact:true}).click();
 const lightbox=page.locator('.solara-lightbox-dialog');
 await lightbox.waitFor({state:'visible'});
 await lightbox.getByRole('button',{name:'Zoom in',exact:true}).click();
 assert.equal((await lightbox.locator('.zoom-level-btn').textContent()).trim(),'125%');
 await lightbox.getByRole('button',{name:'Reset zoom',exact:true}).click();
 assert.equal((await lightbox.locator('.zoom-level-btn').textContent()).trim(),'100%');
 await lightbox.getByRole('button',{name:'Copy screenshot',exact:true}).click();
 await lightbox.getByRole('button',{name:'Move screenshot to Trash',exact:true}).click();
 await page.getByRole('dialog').getByText("will move to this instance's Trash",{exact:false}).waitFor();
 await page.getByRole('dialog').getByRole('button',{name:'Cancel'}).click();

 // Exercise instance duplication dialog and feedback
 await page.getByRole('button',{name:'Create similar instance',exact:true}).click();
 const dupModal=page.getByRole('dialog').filter({hasText:'Duplicate an instance'});
 await dupModal.waitFor({state:'visible'});
 await dupModal.locator('input').fill('Cloned QA Adventure');
 await dupModal.getByRole('button',{name:'Duplicate instance',exact:true}).click();
 await dupModal.waitFor({state:'hidden'});
 assert.equal(await page.evaluate(()=>window.__WORKSPACE_QA__.calls.some(c=>c.command==='duplicate_instance'&&c.args.name==='Cloned QA Adventure')),true);

 // Exercise console log viewer ergonomics: search match count, export logs action, and pill counts
 await page.getByRole('navigation',{name:'Instance pages'}).getByRole('button',{name:'Logs',exact:true}).click();
 await page.getByRole('button',{name:'Export logs',exact:true}).waitFor();
 await page.locator('.console-search-wrap input').fill('QA');
 await page.locator('.search-match-badge').waitFor();
 assert.equal(await page.locator('.search-match-badge').textContent(),'1 match');
 assert.equal(await page.locator('.pill-count').first().isVisible(),true);
 // Verify regex bracket character search safety
 await page.locator('.console-search-wrap input').fill('[');
 assert.ok(await page.locator('.search-match-badge').textContent());
 await page.locator('.console-search-wrap input').fill('');
 // Trigger export logs
 await page.getByRole('button',{name:'Export logs',exact:true}).click();
 await page.locator('.workspace-message').filter({hasText:/Exported logs to/}).waitFor();

 // Exercise instance notebook: autosave, pulse indicator, and starter templates
 await page.getByRole('navigation',{name:'Instance pages'}).getByRole('button',{name:'Notes',exact:true}).click();
 await page.locator('.badge-saved, .badge-autosave').waitFor();
 assert.equal((await page.locator('.notes-status-badge').textContent()).includes('Saved locally'),true);
 await page.getByRole('button',{name:'+ Server IP & notes',exact:true}).first().click();
 let noteVal=await page.locator('.notes-textarea').inputValue();
 assert.equal(noteVal.includes('Server Details'),true);
 await page.getByRole('button',{name:'+ To-do list',exact:true}).first().click();
 noteVal=await page.locator('.notes-textarea').inputValue();
 assert.equal(noteVal.includes('To-Do List'),true);

 // Exercise Global Keyboard Shortcuts (Ctrl+1..4, Ctrl+,, Ctrl+D, and ? modal)
 await page.keyboard.press('Control+1');
 await page.locator('.toolbar-current strong').filter({hasText:'Home'}).waitFor();
 await page.keyboard.press('Control+3');
 await page.locator('.toolbar-current strong').filter({hasText:'Services'}).waitFor();
 await page.keyboard.press('Control+4');
 await page.locator('.toolbar-current strong').filter({hasText:'Discover'}).waitFor();
 await page.keyboard.press('Control+,');
 await page.locator('.toolbar-current strong').filter({hasText:'Settings'}).waitFor();
 await page.keyboard.press('Control+2');
 await page.locator('.toolbar-current strong').filter({hasText:'Library'}).waitFor();

 // Exercise ? Shortcuts Cheatsheet modal with two-column layout
 await page.keyboard.press('?');
 const shortcutsModal=page.getByRole('dialog').filter({hasText:'Keyboard shortcuts'});
 await shortcutsModal.waitFor({state:'visible'});
 await shortcutsModal.getByText('Navigation',{exact:true}).waitFor();
 await shortcutsModal.getByText('Actions & Search',{exact:true}).waitFor();
 await shortcutsModal.getByText('Home / Dashboard',{exact:true}).waitFor();
 await shortcutsModal.getByText('Toggle downloads queue',{exact:true}).waitFor();
 await shortcutsModal.getByRole('button',{name:'Got it',exact:true}).click();
 await shortcutsModal.waitFor({state:'hidden'});

 // Exercise Downloads Modal toggle via Ctrl+D and Queue Management
 await page.evaluate(()=>{
   window.__WORKSPACE_QA__.downloads=[
     {id:'dl-1',name:'Sodium Fabric 1.21.1',status:'completed',detail:'Minecraft files are ready',downloaded:1048576,total:1048576},
     {id:'dl-2',name:'Iris Shaders',status:'failed',detail:'Could not install',actions:['retry']},
     {id:'dl-3',name:'Fabric Loader 0.16.9',status:'running',detail:'Downloading',speed:2097152,downloaded:524288,total:1048576,actions:['cancel']}
   ];
 });
 await page.keyboard.press('Control+d');
 const downloadsModal=page.getByRole('dialog').filter({hasText:'Your downloads'});
 await downloadsModal.waitFor({state:'visible'});
 await downloadsModal.locator('.queue-count-pill').getByText('1 in queue',{exact:false}).waitFor();
 await downloadsModal.locator('.speed-pill').getByText('2.0 MiB/s',{exact:false}).waitFor();
 // Exercise Clear completed downloads
 await downloadsModal.getByRole('button',{name:'Clear completed',exact:true}).click();
 assert.equal(await page.evaluate(()=>window.__WORKSPACE_QA__.calls.some(c=>c.command==='clear_finished_downloads')),true);
 await page.keyboard.press('Control+d');
 await downloadsModal.waitFor({state:'hidden'});

 // Return to instance QA test world
 await page.getByRole('button',{name:'Open instance QA test world',exact:true}).click();

 // Return to mods tab for running locks test
 await page.getByRole('navigation',{name:'Instance pages'}).getByRole('button',{name:/^Mods/}).click();
 await page.evaluate(()=>window.__WORKSPACE_QA__.instance.running=true);await page.getByRole('button',{name:'Stop game',exact:true}).waitFor();assert.equal(await page.getByRole('switch',{name:'Enable QA mod',exact:true}).isDisabled(),true);assert.equal(await page.getByRole('button',{name:'Add .jar file',exact:true}).isDisabled(),true);
 await page.setViewportSize({width:800,height:700});assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);await page.screenshot({path:'checks/instance-workspace-fixture.png',animations:'disabled'});
 assert.deepEqual(errors,[]);console.log('Instance workspace edit/save, unsaved navigation, rename/trash/restore, backups, local .jar mod import, mod inspector & toggle, keyboard shortcuts, downloads queue, console logs, notebook templates and running locks PASS. Explicit IPC fixtures only.');
}finally{await browser.close()}})().catch(e=>{console.error(e);process.exitCode=1});
