const {chromium}=require('playwright');
const fs=require('node:fs/promises');
const path=require('node:path');
async function settleArt(page){await page.locator('.image-art img').evaluateAll(imgs=>Promise.all(imgs.map(img=>{img.loading='eager';return img.decode().catch(()=>{})})));await page.waitForFunction(()=>[...document.querySelectorAll('.image-art')].every(el=>el.classList.contains('is-loaded')||el.classList.contains('is-failed')))}
(async()=>{
 const base=process.env.SOLARA_UI_URL||'http://127.0.0.1:1420';
 const api=process.env.SOLARA_PREVIEW_API;
 if(api&&!/^http:\/\/(127\.0\.0\.1|localhost):\d+$/.test(api))throw Error('Preview API must be an explicit local development endpoint');
 const url=base+'/desktop.html'+(api?'?api='+encodeURIComponent(api):'');
 const browser=await chromium.launch({headless:true,args:process.env.CI?[]:['--no-sandbox']});
 try{
  await fs.mkdir('public/screenshots',{recursive:true});await fs.mkdir('checks',{recursive:true});
  const context=await browser.newContext({viewport:{width:1440,height:1000}});
  const page=await context.newPage();await page.goto(url,{waitUntil:'networkidle'});
  await page.locator('.boot-screen').waitFor({state:'hidden'});
  if(api)await page.locator('.listing-card').first().waitFor();
  await settleArt(page);
  await page.screenshot({path:'public/screenshots/dashboard.png',animations:'disabled'});
  await page.getByRole('button',{name:'Minecraft launcher',exact:true}).click();await page.waitForTimeout(500);
  await settleArt(page);
  await page.screenshot({path:'public/screenshots/library.png',animations:'disabled'});
  await page.getByRole('button',{name:'Create your first instance',exact:true}).click();
  await page.getByRole('radio',{name:'Fabric',exact:true}).click();await page.waitForTimeout(400);
  await page.screenshot({path:'checks/instance-dialog.png',animations:'disabled'});
  await page.keyboard.press('Escape');await page.getByRole('dialog').waitFor({state:'hidden'});
  await page.getByRole('button',{name:'Settings',exact:true}).click();await page.waitForTimeout(450);
  await page.screenshot({path:'checks/settings-redesign.png',animations:'disabled'});
  await page.goto(base,{waitUntil:'networkidle'});await settleArt(page);
  await page.screenshot({path:'checks/website-hero.png',animations:'disabled'});
  await page.screenshot({path:'checks/website-desktop.png',fullPage:true,animations:'disabled'});
  await page.setViewportSize({width:390,height:844});await page.screenshot({path:'checks/website-mobile.png',fullPage:true,animations:'disabled'});
  await context.close();
  const videoContext=await browser.newContext({viewport:{width:1280,height:880},recordVideo:{dir:'checks/recordings',size:{width:1280,height:880}}});
  const motion=await videoContext.newPage();await motion.goto(url,{waitUntil:'networkidle'});
  await motion.locator('.boot-screen').waitFor({state:'hidden'});await settleArt(motion);await motion.waitForTimeout(1200);await motion.getByRole('button',{name:'Open Minecraft',exact:true}).hover();await motion.waitForTimeout(650);
  await motion.getByRole('button',{name:'Open Minecraft',exact:true}).click();await motion.waitForTimeout(1500);
  await motion.getByRole('button',{name:'Create your first instance',exact:true}).click();await motion.waitForTimeout(600);
  for(const name of ['Fabric','Forge','NeoForge']){await motion.getByRole('radio',{name,exact:true}).hover();await motion.getByRole('radio',{name,exact:true}).click();await motion.waitForTimeout(350)}
  await motion.keyboard.press('Escape');await motion.getByRole('dialog').waitFor({state:'hidden'});
  await motion.getByRole('button',{name:'Your servers',exact:true}).hover();await motion.waitForTimeout(450);await motion.getByRole('button',{name:'Your servers',exact:true}).click();await motion.waitForTimeout(600);
  await settleArt(motion);await motion.waitForTimeout(800);await motion.getByRole('button',{name:'Explore mods and modpacks',exact:true}).click();await settleArt(motion);await motion.waitForTimeout(1100);
  await motion.getByRole('button',{name:'Dashboard',exact:true}).click();await motion.waitForTimeout(1500);
  const video=motion.video();await videoContext.close();await video.saveAs('checks/solara-motion.webm');
  await fs.writeFile('checks/preview-info.json',JSON.stringify({capturedAt:new Date().toISOString(),mode:api?'real local API development preview':'guest development preview',api:api||null,artwork:'Original illustrative voxel landscapes, generated with built-in imagegen and optimized as local WebP; not actual game screenshots',video:'checks/solara-motion.webm',productionDeployment:false},null,2));
  console.log('Updated actual UI screenshots and recorded the animation preview.');
 }finally{await browser.close()}
})().catch(e=>{console.error(e);process.exitCode=1});
