// Explicit browser-only fixtures. These checks never provision DNS or install a game.
const {chromium}=require('playwright');
const assert=require('node:assert/strict');
const fs=require('node:fs/promises');
(async()=>{
 const base=process.env.SOLARA_UI_URL||'http://127.0.0.1:1420';
 const browser=await chromium.launch({headless:true,args:process.env.CI?[]:['--no-sandbox']});
 const errors=[];
 try{
  const context=await browser.newContext({viewport:{width:1280,height:900}});
  await context.route('**/v1/launcher/bootstrap',r=>r.fulfill({status:503,body:'Explicit QA API outage'}));
  await context.addInitScript(()=>{
   const original={id:'a'.repeat(32),serverId:'12345678-1234-1234-1234-123456789abc',name:'qa-world',hostname:'qa-world.games.solara-inc.in',endpoint:'8.8.8.8',port:25565,status:'active',lastError:null,updatedAt:'2026-10-29T00:00:00Z'};
   const state=window.__SOLARA_QA__={calls:[],instances:[],downloads:[],assignments:[original],owner:'qa-owner-a',failPatch:false,delayList:false,resolveList:null};
   window.__TAURI_INTERNALS__={transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(command,args={})=>{
    state.calls.push({command,args});
    if(command==='plugin:event|listen')return 1;
    if(command==='initialize_launcher')return null;
    if(command==='launcher_capabilities')return {loaders:['vanilla','fabric','forge','quilt','neoforge'],contentInstall:true,preview:false};
    if(command==='launcher_settings')return {memory:4096,javaPath:'java',discord:false};
    if(command==='launcher_snapshot')return structuredClone({instances:state.instances,localServers:[{id:original.serverId,name:'QA local server',version:'1.21.1',state:'stopped',port:25565}],downloads:state.downloads,minecraftAccounts:[]});
    if(command==='install_content'&&args.contentType!=='modpack'){state.downloads=[{id:'qa-content-job',name:'QA content installation',status:'running',detail:'Downloading and installing content…',actions:[]}];return null}
    if(command==='solara_profile')return {id:state.owner,name:'QA fixture user'};
    if(command==='solara_sign_out'){state.owner=null;return null}
    if(command==='solara_sign_in'){state.owner='qa-owner-b';return {id:state.owner,name:'QA fixture second user'}}
    if(command==='solara_request'){
     if(args.method==='GET'){
      const assignments=state.owner==='qa-owner-a'?structuredClone(state.assignments):[];
      if(state.delayList){state.delayList=false;return new Promise(resolve=>state.resolveList=()=>resolve({assignments}))}
      return {assignments};
     }
     if(args.method==='PATCH'){
      if(state.failPatch)throw Error('Verify control of this public endpoint before updating its address.');
      Object.assign(state.assignments[0],{endpoint:args.body.endpoint,port:args.body.port});return {status:'active'};
     }
     if(args.method==='DELETE'){state.assignments=[];return {released:true}}
     if(args.method==='POST')throw Error('Verify control of this public endpoint before requesting an address.');
    }
    return null;
   }};
  });
  const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));
  let failVersions=true;
  await page.route('https://api.modrinth.com/v2/search?**',route=>{
   const query=new URL(route.request().url());const facets=JSON.parse(query.searchParams.get('facets'));const kind=facets[0][0].split(':')[1];
   return route.fulfill({json:{hits:[{project_id:kind==='modpack'?'qa-pack':'qa-mod',slug:kind==='modpack'?'qa-pack':'qa-mod',project_type:kind,title:kind==='modpack'?'QA fixture modpack':'QA fixture mod',description:'Explicit browser test fixture',author:'QA fixture author',downloads:12}]}});
  });
  await page.route('https://api.modrinth.com/v2/project/qa-pack/version',route=>{
   if(failVersions){failVersions=false;return route.fulfill({status:503,body:'QA provider failure'})}
   const version=(id,type,date)=>({id,project_id:'qa-pack',name:'QA '+type,version_type:type,date_published:date,game_versions:['1.21.1'],loaders:['fabric'],files:[{filename:'qa.mrpack'}]});
   return route.fulfill({json:[version('qa-beta','beta','2026-10-29'),version('qa-release','release','2026-10-28')]});
  });
  await page.goto(base+'/desktop.html',{waitUntil:'networkidle'});await page.locator('.boot-screen').waitFor({state:'hidden'});
  await page.getByRole('button',{name:'Explore mods and modpacks',exact:true}).click();
  const chooseType=async name=>{await page.getByRole('combobox',{name:'Content type',exact:true}).click();await page.getByRole('option',{name,exact:true}).click()};
  await chooseType('Modpacks');await page.getByRole('button',{name:'Search',exact:true}).click();
  await page.getByRole('heading',{name:'QA fixture modpack',exact:true}).waitFor();
  await page.getByRole('button',{name:'Install',exact:true}).click();
  await page.getByRole('heading',{name:'Install a modpack',exact:true}).waitFor();
  await page.getByRole('alert').filter({hasText:'Content provider returned 503'}).waitFor();
  assert.equal(await page.getByRole('button',{name:'Install modpack',exact:true}).isDisabled(),true);
  await page.getByRole('button',{name:'Try again',exact:true}).click();
  await page.getByRole('combobox',{name:'Modpack version',exact:true}).waitFor();
  assert.match(await page.getByRole('combobox',{name:'Modpack version',exact:true}).innerText(),/QA release/,'Stable release is the default even when a beta is newer');
  await page.getByRole('combobox',{name:'Modpack version',exact:true}).click();
  await page.getByRole('option',{name:/QA beta/}).click();
  await fs.mkdir('checks',{recursive:true});await page.screenshot({path:'checks/qa-modpack-flow.png',animations:'disabled'});
  await page.getByRole('button',{name:'Install modpack',exact:true}).click();await page.getByRole('dialog').waitFor({state:'hidden'});
  const pack=await page.evaluate(()=>window.__SOLARA_QA__.calls.find(c=>c.command==='install_content'));
  assert.deepEqual(pack.args,{instanceId:null,projectId:'qa-pack',contentType:'modpack',versionId:'qa-beta'});
  // Mods still require a real selected target; this test supplies only explicitly labelled IPC fixtures.
  await page.evaluate(()=>{window.__SOLARA_QA__.instances=[{id:'qa-first',name:'QA first instance',gameVersion:'1.21.1',loader:'fabric',state:'installed',running:false},{id:'qa-second',name:'QA second instance',gameVersion:'1.21.1',loader:'fabric',state:'installed',running:false}]});
  await page.getByRole('heading',{name:'QA second instance',exact:true}).waitFor();
  await page.getByRole('textbox',{name:'Search your library',exact:true}).fill('QA second');
  assert.equal(await page.locator('.instance-collection .instance-card').count(),1,'Library search filters actual loaded instance data');
  await page.getByRole('button',{name:'Show list view',exact:true}).click();
  assert.equal(await page.locator('.instance-collection').evaluate(e=>e.classList.contains('as-list')),true);
  assert.equal(await page.evaluate(()=>localStorage.getItem('solara:library-view')),'list');
  await page.getByRole('textbox',{name:'Search your library',exact:true}).fill('nothing matches');
  await page.getByRole('heading',{name:'No instances match',exact:true}).waitFor();
  await page.getByRole('button',{name:'Clear search',exact:true}).click();
  assert.equal(await page.locator('.instance-collection .instance-card').count(),2);
  await page.getByRole('button',{name:'Show grid view',exact:true}).click();

  await page.getByRole('button',{name:'Explore mods and modpacks',exact:true}).click();await chooseType('Mods');
  assert.equal(await page.getByRole('heading',{name:'QA fixture modpack',exact:true}).count(),0,'Changing type removes stale results');
  await page.getByRole('button',{name:'Search',exact:true}).click();await page.getByRole('button',{name:'Install',exact:true}).click();
  await page.getByRole('combobox',{name:'Install into',exact:true}).click();await page.getByRole('option',{name:/QA second instance/}).click();
  await page.getByRole('button',{name:'Install content',exact:true}).click();await page.getByRole('dialog').waitFor({state:'hidden'});
  const mod=await page.evaluate(()=>window.__SOLARA_QA__.calls.filter(c=>c.command==='install_content').at(-1));assert.equal(mod.args.instanceId,'qa-second');assert.equal(mod.args.contentType,'mod');
  await page.getByRole('button',{name:'Downloads',exact:true}).click();
  await page.getByRole('progressbar',{name:'Installing QA content installation',exact:true}).waitFor();
  await page.evaluate(()=>{window.__SOLARA_QA__.downloads[0].status='completed';window.__SOLARA_QA__.downloads[0].detail='Content installed.'});
  await page.getByText('Content installed.',{exact:true}).waitFor();
  assert.equal(await page.getByRole('progressbar',{name:'Installing QA content installation',exact:true}).count(),0);
  await page.getByRole('button',{name:'Close dialog',exact:true}).click();await page.getByRole('dialog').waitFor({state:'hidden'});
  await page.getByRole('button',{name:'Solara services',exact:true}).click();await page.getByRole('button',{name:'Hosting',exact:true}).click();
  await page.getByRole('heading',{name:'qa-world.games.solara-inc.in',exact:true}).waitFor();
  await page.getByRole('button',{name:'Request a subdomain',exact:true}).click();
  await page.getByLabel('Requested name',{exact:true}).fill('qa-new-world');await page.getByLabel('Public IP address',{exact:true}).fill('8.8.4.4');
  await page.getByRole('checkbox',{name:'I control this endpoint and understand its IP address becomes public.',exact:true}).check();
  await page.getByRole('button',{name:'Request assignment',exact:true}).click();
  await page.getByRole('dialog').getByRole('alert').filter({hasText:'Verify control'}).waitFor();
  await page.getByRole('button',{name:'Close dialog',exact:true}).click();await page.getByRole('dialog').waitFor({state:'hidden'});
  await page.getByRole('button',{name:'Update endpoint',exact:true}).click();
  await page.getByLabel('Public IP address',{exact:true}).fill('8.8.4.4');await page.getByLabel('Public port',{exact:true}).fill('25566');
  await page.getByRole('checkbox',{name:'I control this endpoint and understand its IP address becomes public.',exact:true}).check();
  await page.evaluate(()=>window.__SOLARA_QA__.failPatch=true);
  await page.getByRole('dialog').getByRole('button',{name:'Update endpoint',exact:true}).click();
  await page.getByRole('dialog').getByRole('alert').filter({hasText:'Verify control'}).waitFor();
  await page.evaluate(()=>window.__SOLARA_QA__.failPatch=false);
  await page.getByRole('dialog').getByRole('button',{name:'Update endpoint',exact:true}).click();await page.getByRole('dialog').waitFor({state:'hidden'});
  await page.getByText('8.8.4.4:25566',{exact:true}).waitFor();
  await page.screenshot({path:'checks/qa-address-management.png',animations:'disabled'});
  const patch=await page.evaluate(()=>window.__SOLARA_QA__.calls.filter(c=>c.command==='solara_request'&&c.args.method==='PATCH').at(-1));
  assert.deepEqual(patch.args.body,{endpoint:'8.8.4.4',port:25566,consent:true});
  await page.getByRole('button',{name:'Release address',exact:true}).click();
  assert.equal(await page.getByRole('dialog').getByRole('button',{name:'Release address',exact:true}).isDisabled(),true);
  assert.equal(await page.evaluate(()=>window.__SOLARA_QA__.calls.some(c=>c.command==='solara_request'&&c.args.method==='DELETE')),false);
  await page.getByRole('checkbox',{name:'I understand this removes the server address.',exact:true}).check();
  await page.getByRole('dialog').getByRole('button',{name:'Release address',exact:true}).click();await page.getByRole('dialog').waitFor({state:'hidden'});
  await page.getByRole('heading',{name:'No server addresses yet',exact:true}).waitFor();
  // A delayed private response from an old account must not populate a new account's view.
  await page.evaluate(()=>{const q=window.__SOLARA_QA__;q.assignments=[{id:'b'.repeat(32),serverId:'qa-old',name:'old-owner-secret',hostname:'old-owner-secret.games.solara-inc.in',endpoint:'8.8.8.8',port:25565,status:'active'}];q.delayList=true});
  await page.getByRole('button',{name:'Refresh addresses',exact:true}).click();await page.waitForFunction(()=>typeof window.__SOLARA_QA__.resolveList==='function');
  await page.getByRole('button',{name:'QA fixture user',exact:true}).click();await page.getByRole('button',{name:'Sign out of Solara',exact:true}).click();await page.getByRole('dialog').waitFor({state:'hidden'});
  await page.getByRole('button',{name:'Sign in to Solara',exact:true}).click();await page.getByRole('button',{name:'QA fixture second user',exact:true}).waitFor();
  await page.evaluate(()=>window.__SOLARA_QA__.resolveList());
  await page.getByRole('heading',{name:'No server addresses yet',exact:true}).waitFor();
  assert.equal(await page.getByText('old-owner-secret.games.solara-inc.in',{exact:true}).count(),0);
  assert.deepEqual(errors,[]);
  await fs.writeFile('checks/workflow-results.json',JSON.stringify({passed:true,fixtureOnly:true,firstModpackWithoutInstance:true,explicitVersion:true,stableDefault:true,providerFailureRetry:true,selectedModTarget:true,contentDownloadStates:true,staleSearchCleared:true,assignmentList:true,createRequiresEndpointProof:true,updateFailureRetry:true,explicitReleaseConsent:true,crossAccountStaleResponseIgnored:true,errors,note:'Provider metadata/native IPC/assignment data are labelled QA fixtures; no game install or live DNS operation performed.'},null,2));
  console.log('Content and account-owned address browser workflow fixtures passed. No game install or DNS change performed.');
 }finally{await browser.close()}
})().catch(e=>{console.error(e);process.exitCode=1});
