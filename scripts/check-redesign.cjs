// Browser-only UI/IPC/provider fixtures. These do not install or launch Minecraft.
const {chromium}=require('playwright');
const assert=require('node:assert/strict');
const fs=require('node:fs/promises');
const path=require('node:path');
const base=process.env.SOLARA_UI_URL||'http://127.0.0.1:1420';
const output=path.resolve('checks/redesign');

(async()=>{
  await fs.mkdir(output,{recursive:true});
  const browser=await chromium.launch({headless:true,args:['--no-sandbox']});
  const errors=[];
  try {
    const guest=await browser.newContext({viewport:{width:1440,height:1000}});
    await guest.route('**/v1/launcher/bootstrap',r=>r.fulfill({status:503,body:'Explicit QA outage'}));
    const gp=await guest.newPage();gp.on('pageerror',e=>errors.push(e.message));
    await gp.goto(base+'/desktop.html',{waitUntil:'networkidle'});
    await gp.locator('.boot-screen').waitFor({state:'hidden'});
    await gp.screenshot({path:path.join(output,'home-guest.png')});
    assert.equal(await gp.getByRole('button',{name:'Sign in to Solara',exact:true}).isDisabled(),true,'Browser preview must not pretend native sign-in works');
    await gp.getByRole('button',{name:'Minecraft launcher',exact:true}).click();
    await gp.mouse.move(900,60);
    await gp.screenshot({path:path.join(output,'library-guest.png'),animations:'disabled'});
    await guest.close();

    const context=await browser.newContext({viewport:{width:1440,height:1000}});
    await context.route('https://mc-heads.net/**',r=>r.abort());
    await context.route('**/v1/launcher/bootstrap',r=>r.fulfill({json:{schemaVersion:1,events:[],projects:[],services:[
      {id:'qa-hosting',name:'QA · Hosting',description:'Explicit service fixture',category:'hosting',status:'operational',url:'https://www.solara-inc.in',kind:'web'},
      {id:'qa-tools',name:'QA · Tools',description:'Explicit service fixture',category:'tools',status:'operational',url:'https://www.solara-inc.in',kind:'web'}
    ],releases:[]}}));
    await context.route('https://api.modrinth.com/v2/search?**',r=>{
      const facets=JSON.parse(new URL(r.request().url()).searchParams.get('facets'));
      const type=facets[0][0].split(':')[1];
      return r.fulfill({json:{hits:[
        {project_id:'qa-project-a',slug:'qa-project-a',title:'QA · A new perspective',author:'Preview creator',description:'A browser fixture for checking search results, readable descriptions and content actions.',project_type:type,downloads:1200},
        {project_id:'qa-project-b',slug:'qa-project-b',title:'QA · The little details',author:'Preview creator',description:'A second explicit test project for verifying the redesigned discovery list.',project_type:type,downloads:2300}
      ]}});
    });
    await context.addInitScript(()=>{
      const state=window.__REDESIGN_QA__={calls:[],backups:[],downloads:[],update:{state:'development'},files:{'options.txt':{content:'QA file contents',sha256:'qa-original'}},instances:[
        {id:'qa-evergreen',name:'QA · Evergreen',gameVersion:'1.21.1',loader:'fabric',loaderVersion:'0.20.1',state:'installed',running:false,memory:4096,javaPath:'auto'},
        {id:'qa-coastal',name:'QA · Coastal creative',gameVersion:'1.21.1',loader:'vanilla',loaderVersion:'',state:'installed',running:false,memory:4096,javaPath:'auto'},
        {id:'qa-workshop',name:'QA · Workshop',gameVersion:'1.20.1',loader:'forge',loaderVersion:'47.4.28',state:'installed',running:false,memory:6144,javaPath:'auto'}
      ]};
      window.__TAURI_INTERNALS__={transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(command,args={})=>{
        state.calls.push({command,args});
        if(command==='plugin:event|listen')return 1;
        if(command==='launcher_capabilities')return {loaders:['vanilla','fabric','quilt','forge','neoforge'],contentInstall:true,automaticJava:true,preview:false};
        if(command==='launcher_settings')return {memory:4096,javaPath:'auto',discord:true};
        if(command==='launcher_snapshot')return structuredClone({instances:state.instances,minecraftAccounts:[{id:'qa-profile',name:'QA player',active:true}],localServers:[],downloads:state.downloads});
        if(command==='solara_profile')return {id:'qa-user',name:'QA player'};
        if(command==='solara_request')return {assignments:[],servers:[]};
        if(command==='app_update_status')return structuredClone(state.update);
        if(command==='minecraft_versions')return [{id:'1.21.1',type:'release'},{id:'1.20.1',type:'release'}];
        if(command==='loader_versions')return [{id:'0.20.1',stable:true}];
        if(command==='instance_mods')return [
          {path:'mods/qa-a.jar',fileName:'qa-a.jar',name:'QA · Better horizons',version:'1.0',enabled:true,size:412000,loaders:['fabric']},
          {path:'mods/qa-b.jar',fileName:'qa-b.jar',name:'QA · A few little details',version:'2.0',enabled:true,size:81000,loaders:['fabric']}
        ];
        if(command==='instance_files'&&args.path==='screenshots')return ['QA first.png','QA second.png'].map((name,i)=>({name,path:'screenshots/'+name,directory:false,size:1234,modifiedAt:1780000000-i}));
        if(command==='instance_image')return 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+a3ioAAAAASUVORK5CYII=';
        if(command==='instance_files')return Object.entries(state.files).map(([path,file])=>({path,name:path,directory:false,size:file.content.length}));
        if(command==='instance_text')return {...state.files[args.path],path:args.path};
        if(command==='instance_backups')return state.backups;
        if(command==='instance_backup_action'){
          const entry={id:'qa-backup',name:args.name,size:1024,fileCount:1,worldCount:0,createdAt:new Date().toISOString()};
          state.backups.push(entry);return entry;
        }
        if(command==='instance_logs')return '[INFO] Explicit QA console fixture\n[WARN] This is a browser preview, no game is running.\n[ERROR] Example fixture message for severity filtering';
        return null;
      }};
    });
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));
    await page.goto(base+'/desktop.html',{waitUntil:'networkidle'});
    await page.locator('.boot-screen').waitFor({state:'hidden'});
    await page.getByRole('heading',{name:'Welcome back, QA player.',exact:true}).waitFor();
    const rail=await page.locator('.sidebar').boundingBox();
    const toolbar=await page.locator('.app-topbar').boundingBox();
    const content=await page.locator('.app-content').boundingBox();
    assert.ok(rail.width>=76&&rail.width<=100,'Larger labelled icon rail stays compact');
    assert.ok(toolbar.y+toolbar.height<=content.y+1,'Toolbar shares the titlebar row rather than adding a second row');
    assert.equal(await page.getByRole('complementary',{name:'Your accounts and services',exact:true}).isVisible(),true);
    await page.screenshot({animations:'disabled',path:path.join(output,'home-populated-fixture.png')});

    // Global search must actually transfer the query to discovery.
    await page.getByRole('textbox',{name:'Search all content',exact:true}).fill('adventure');
    await page.getByRole('textbox',{name:'Search all content',exact:true}).press('Enter');
    await page.getByRole('heading',{name:'QA · A new perspective',exact:true}).waitFor();
    assert.equal(await page.getByRole('textbox',{name:'Search mods, modpacks and more',exact:true}).inputValue(),'adventure');
    await page.screenshot({animations:'disabled',path:path.join(output,'discover-fixture.png')});

    await page.getByRole('button',{name:'Minecraft launcher',exact:true}).click();
    assert.equal(await page.getByRole('complementary',{name:'Your accounts and services',exact:true}).isVisible(),true,'Account column persists when opening the library');
    await page.getByRole('button',{name:'Forge',exact:true}).click();
    assert.equal(await page.locator('.instance-collection .instance-card').count(),1);
    await page.getByRole('heading',{name:'QA · Workshop',exact:true}).waitFor();
    await page.getByRole('button',{name:'All instances',exact:true}).click();
    await page.screenshot({animations:'disabled',path:path.join(output,'library-fixture.png')});

    // New instance shortcuts must open the right workspace and load its mods.
    await page.getByRole('button',{name:'Open sidebar instance QA · Evergreen',exact:true}).click();
    await page.getByText('QA · Better horizons',{exact:true}).waitFor();
    await page.screenshot({animations:'disabled',path:path.join(output,'instance-mods-fixture.png')});
    const nav=page.getByRole('navigation',{name:'Instance pages'});
    await nav.getByRole('button',{name:'Screenshots',exact:true}).click();
    await page.getByRole('button',{name:'View screenshot QA first.png',exact:true}).getByRole('img').waitFor();
    await page.getByRole('button',{name:'View screenshot QA first.png',exact:true}).focus();
    await page.keyboard.press('Enter');
    const lightbox=page.getByRole('dialog',{name:'Screenshot viewer',exact:true});
    assert.equal(await lightbox.evaluate(e=>e.matches(':modal')),true);
    await page.keyboard.press('ArrowRight');
    await lightbox.getByText('QA second.png',{exact:true}).waitFor();
    await page.keyboard.press('Escape');await lightbox.waitFor({state:'hidden'});
    await nav.getByRole('button',{name:'Files',exact:true}).click();
    await page.getByRole('button',{name:'options.txt',exact:true}).click();
    await page.getByRole('textbox',{name:'File contents',exact:true}).fill('QA unsaved edit');
    await page.getByRole('button',{name:'Go back',exact:true}).click();
    await page.getByRole('dialog').getByRole('button',{name:'Cancel',exact:true}).click();
    assert.equal(await page.getByRole('textbox',{name:'File contents',exact:true}).inputValue(),'QA unsaved edit','History must preserve an unsaved file when navigation is cancelled');
    await page.getByRole('button',{name:'Close file editor',exact:true}).click();
    await page.getByRole('dialog').getByRole('button',{name:'Discard changes',exact:true}).click();
    await page.getByRole('button',{name:'Create world snapshot',exact:true}).click();
    const snapshot=page.getByRole('dialog',{name:'Save a world snapshot',exact:true});
    assert.equal(await snapshot.evaluate(e=>e.matches(':modal')),true,'Snapshot uses a real modal dialog');
    await snapshot.getByRole('button',{name:'Save snapshot',exact:true}).click();
    await snapshot.getByRole('status').waitFor();
    assert.equal(await page.evaluate(()=>window.__REDESIGN_QA__.calls.filter(c=>c.command==='instance_backup_action').length),1);
    await page.keyboard.press('Escape');await snapshot.waitFor({state:'hidden'});
    await nav.getByRole('button',{name:'Settings',exact:true}).click();
    await page.screenshot({animations:'disabled',path:path.join(output,'instance-settings-fixture.png')});
    // Personal icons are local, validated, persistent, and shared across the shell.
    assert.equal(await page.locator('.instance-loader-badge img').count(),0,'Default instance icon is a vector loader glyph');
    const iconInput=page.getByLabel('Upload instance icon',{exact:true});
    await iconInput.setInputFiles({name:'unsafe.svg',mimeType:'image/svg+xml',buffer:Buffer.from('<svg/>')});
    await page.getByRole('alert').filter({hasText:'Choose a PNG, JPG or WebP image.'}).waitFor();
    const iconBase64=await page.evaluate(()=>{const c=document.createElement('canvas');c.width=100;c.height=100;const ctx=c.getContext('2d');ctx.fillStyle='#ea8da9';ctx.fillRect(0,0,100,100);ctx.fillStyle='#ffda94';ctx.fillRect(25,25,50,50);return c.toDataURL('image/png').split(',')[1]});
    await iconInput.setInputFiles({name:'my-qa-icon.png',mimeType:'image/png',buffer:Buffer.from(iconBase64,'base64')});
    await page.getByRole('status').filter({hasText:'Your icon is saved on this device.'}).waitFor();
    await page.locator('.instance-loader-badge img').waitFor();
    await page.screenshot({animations:'disabled',path:path.join(output,'custom-icon-fixture.png')});
    await page.reload({waitUntil:'networkidle'});await page.locator('.boot-screen').waitFor({state:'hidden'});
    await page.getByRole('button',{name:'Open sidebar instance QA · Evergreen',exact:true}).locator('img').waitFor();
    await page.getByRole('button',{name:'Open sidebar instance QA · Evergreen',exact:true}).click();
    await page.getByRole('navigation',{name:'Instance pages'}).getByRole('button',{name:'Settings',exact:true}).click();
    await page.getByRole('region',{name:'Instance icon'}).getByRole('button',{name:'Reset',exact:true}).click();
    assert.equal(await page.locator('.instance-loader-badge img').count(),0,'Reset restores the loader glyph everywhere');


    await page.locator('.sidebar').getByRole('button',{name:'Settings',exact:true}).click();
    await page.screenshot({animations:'disabled',path:path.join(output,'settings-fixture.png')});
    await page.getByRole('button',{name:'Solara services',exact:true}).click();
    await page.screenshot({animations:'disabled',path:path.join(output,'services-fixture.png')});
    await page.getByRole('button',{name:'Create Minecraft instance',exact:true}).click();
    await page.getByRole('dialog',{name:'Create an instance',exact:true}).waitFor();
    await page.waitForFunction(()=>getComputedStyle(document.querySelector('.modal-shade')).opacity==='1');
    await page.screenshot({animations:'disabled',path:path.join(output,'create-instance-fixture.png')});
    await page.getByRole('button',{name:'Close dialog',exact:true}).click();

    // Quick starts are honest loader setups and carry native-compatible args.
    await page.getByRole('button',{name:'Dashboard',exact:true}).click();
    await page.getByRole('button',{name:'Quick starts',exact:true}).click();
    assert.equal(await page.locator('.preset-card img').count(),0,'Quick starts also use vector loader icons');
    await page.screenshot({animations:'disabled',path:path.join(output,'quick-starts-fixture.png')});
    await page.locator('.preset-card').filter({hasText:'Fabric essentials'}).getByRole('button',{name:'Create instance',exact:true}).click();
    assert.deepEqual(await page.evaluate(()=>window.__REDESIGN_QA__.calls.find(c=>c.command==='create_instance').args),{name:'Fabric essentials',gameVersion:'1.21.1',loader:'fabric',loaderVersion:''});
    if(await page.getByRole('button',{name:'Dismiss notification',exact:true}).isVisible())await page.getByRole('button',{name:'Dismiss notification',exact:true}).click();

    for(const width of [1440,1100,800,650]){
      await page.setViewportSize({width,height:width===800?620:900});
      for(const name of ['Dashboard','Minecraft launcher','Explore mods and modpacks','Solara services','Settings']){
        await page.locator('.sidebar').getByRole('button',{name,exact:true}).click();
        assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth||document.querySelector('.app-content').scrollWidth>document.querySelector('.app-content').clientWidth+1),false,name+' fits at '+width);
        const footer=await page.locator('.app-footer').boundingBox();
        assert.ok(footer&&footer.y+footer.height<=page.viewportSize().height+1,'Footer remains visible');
        const settings=await page.locator('.sidebar').getByRole('button',{name:'Settings',exact:true}).boundingBox();
        assert.ok(settings&&settings.y+settings.height<=footer.y,'Sidebar settings stays above footer at '+width);
      }
      if(width===800){await page.getByRole('button',{name:'Dashboard',exact:true}).click();await page.mouse.move(700,55);await page.screenshot({animations:'disabled',path:path.join(output,'small-window-fixture.png')});}
    }
    await page.setViewportSize({width:1440,height:1000});
    await page.getByRole('button',{name:'Dashboard',exact:true}).click();
    await page.evaluate(()=>{window.__REDESIGN_QA__.update={state:'downloading',downloaded:3145728,total:6291456};window.__REDESIGN_QA__.downloads=[{id:'qa-download',name:'QA game files',status:'downloading',downloaded:1048576,total:4194304}]});
    await page.waitForFunction(()=>document.querySelector('.app-update-banner progress')?.value===.5);
    assert.equal(await page.locator('.app-update-banner').getByText('50%',{exact:true}).count(),1,'Update percent comes from native bytes');
    assert.equal(await page.locator('.dock-progress').evaluate(e=>e.value),.25,'Download dock uses actual task bytes');
    await page.screenshot({animations:'disabled',path:path.join(output,'update-progress-fixture.png')});
    await page.evaluate(()=>{window.__REDESIGN_QA__.update={state:'checking'};window.__REDESIGN_QA__.downloads=[]});
    await page.waitForFunction(()=>document.querySelector('.app-update-banner progress')&&!document.querySelector('.app-update-banner progress').hasAttribute('value'));
    assert.equal(await page.locator('.app-update-banner').getByText('%').count(),0,'Unknown totals never invent a percent');
    await page.emulateMedia({reducedMotion:'reduce'});
    assert.equal(await page.locator('.section-view').evaluate(e=>getComputedStyle(e).animationName),'none');
    assert.deepEqual(errors,[]);
    await fs.writeFile(path.join(output,'results.json'),JSON.stringify({passed:true,globalSearch:true,libraryFilter:true,sidebarInstance:true,initialModLoad:true,historyUnsavedGuard:true,snapshotModal:true,quickStartNativeArguments:true,localIconUploadPersistenceReset:true,realByteProgress:true,responsiveWidths:[1440,1100,800,650],osReducedMotion:true,errors,note:'Browser-only fixtures. No Windows/game/auth/provider acceptance is implied.'},null,2));
    const pictures=[['home-guest.png','Home · actual browser guest state, API unavailable'],['home-populated-fixture.png','Home · illustrative QA accounts and instances'],['library-fixture.png','Library · QA instance fixtures'],['discover-fixture.png','Discover · QA content provider fixtures'],['instance-mods-fixture.png','Instance mods · QA local IPC fixtures'],['instance-settings-fixture.png','Instance settings · QA local IPC fixtures'],['services-fixture.png','Services · QA catalogue fixtures'],['settings-fixture.png','Settings · browser preferences with QA native values'],['create-instance-fixture.png','Create instance · QA version metadata'],['small-window-fixture.png','800 × 620 window · QA fixtures'],['quick-starts-fixture.png','Quick starts · vector loader symbols'],['custom-icon-fixture.png','Personal image upload · explicit browser QA fixture'],['update-progress-fixture.png','Native byte progress · explicit browser IPC fixture']];
    await fs.writeFile(path.join(output,'index.html'),'<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Solara desktop redesign preview</title><style>body{margin:0;background:#181a22;color:#e3e8f2;font:15px/1.65 system-ui;padding:35px;max-width:1500px;margin:auto}h1{letter-spacing:-1px}p,figcaption{color:#9baabd}figure{margin:32px 0 48px}img{width:100%;border:1px solid #ffffff12;border-radius:13px;margin-top:12px}a{color:#8ccbdd}</style><h1>Solara · a little more colourful.</h1><p>Desktop UI redesign preview. Populated screens use explicit browser QA fixtures. They do not demonstrate a live game, account or hosting integration.</p>'+pictures.map(([file,label])=>'<figure><figcaption>'+label+'</figcaption><a href="'+file+'"><img src="'+file+'" alt="'+label+'" loading="lazy"></a></figure>').join('')+'</html>');
    await context.close();
    console.log('Redesigned search, library filters, sidebar, unsaved history, snapshot modal, quick starts, responsive windows and reduced motion PASS. Browser fixtures only. Preview: checks/redesign/index.html');
  } finally {await browser.close()}
})().catch(error=>{console.error(error);process.exitCode=1});
