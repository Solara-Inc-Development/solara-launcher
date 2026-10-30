<script setup lang="ts">
import {ref,computed,watch,nextTick,onUnmounted} from 'vue';
import {native} from '../../packages/shared/client';
import {ArrowLeft,Box,Play,Square,FolderOpen,FileText,Trash2,Pencil,Save,RefreshCw,Search,Plus,X,Terminal,Settings,Package,RotateCcw,Coffee,Copy,ChevronRight,ChevronLeft,ShieldCheck,Globe,MoreVertical,ExternalLink,Compass,Check,Monitor,Download,Sparkles,Activity} from '../../packages/ui/icons';

const props=withDefaults(defineProps<{instance:any,jobs:any[],accountReady?:boolean,parentBusy?:boolean}>(),{accountReady:false,parentBusy:false});
const emit=defineEmits(['back','play','stop','account','refresh','browse-mods','duplicate']);
const tab=ref('mods'),loading=ref(false),busy=ref(false),error=ref(''),notice=ref('');
const path=ref(''),entries=ref<any[]>([]),mods=ref<any[]>([]),trash=ref<any[]>([]),backups=ref<any[]>([]),query=ref(''),modFilter=ref('all');
const editor=ref<any>(null),text=ref(''),consoleText=ref(''),liveLogs=ref(true),modInput=ref<HTMLInputElement|null>(null);
const logSearch=ref(''),logSeverity=ref<'all'|'error'|'warn'|'info'|'chat'>('all'),wrapLogs=ref(false),autoScroll=ref(true),consoleContainer=ref<HTMLElement|null>(null);
const form=ref({name:'',memory:4096,javaPath:'auto'});
const dialog=ref<string|null>(null),dialogElement=ref<HTMLDialogElement|null>(null),dialogName=ref(''),selected=ref<any>(null);
let leaveAnswer:((value:boolean)=>void)|null=null;let logTimer:ReturnType<typeof setInterval>|null=null;

interface LogLine{id:number;text:string;level:'error'|'warn'|'info'|'chat'|'normal';}
const parsedLogs=computed<LogLine[]>(()=>{
  if(!consoleText.value)return [];
  const lines=consoleText.value.split('\n');
  return lines.map((line,idx)=>{
    let level:LogLine['level']='normal';
    if(/\[ERROR\]|\[FATAL\]|Exception|Error:|Caused by:/i.test(line))level='error';
    else if(/\[WARN(?:ING)?\]/i.test(line))level='warn';
    else if(/<[^>]+>|\[CHAT\]/i.test(line))level='chat';
    else if(/\[INFO\]/i.test(line))level='info';
    return{id:idx,text:line,level};
  });
});
const logCounts=computed(()=>{
  let error=0,warn=0,info=0,chat=0;
  for(const line of parsedLogs.value){
    if(line.level==='error')error++;
    else if(line.level==='warn')warn++;
    else if(line.level==='info')info++;
    else if(line.level==='chat')chat++;
  }
  return{total:parsedLogs.value.length,error,warn,info,chat};
});
const visibleLogLines=computed(()=>{
  let lines=parsedLogs.value;
  if(logSeverity.value!=='all'){
    lines=lines.filter(l=>l.level===logSeverity.value);
  }
  if(logSearch.value.trim()){
    const q=logSearch.value.trim().toLowerCase();
    lines=lines.filter(l=>l.text.toLowerCase().includes(q));
  }
  return lines;
});
async function copyFilteredLogs(){
  const content=visibleLogLines.value.map(l=>l.text).join('\n');
  await copy(content||consoleText.value);
}
function clearLogView(){
  consoleText.value='';
}

const dirty=computed(()=>!!editor.value&&text.value!==editor.value.content);
const task=computed(()=>props.jobs.find(j=>j.instanceId===props.instance.id&&['queued','running'].includes(j.status)));
const locked=computed(()=>busy.value||props.parentBusy||props.instance.running||!!task.value);
const visibleFiles=computed(()=>entries.value.filter(f=>f.name.toLowerCase().includes(query.value.toLowerCase())));
const visibleMods=computed(()=>mods.value.filter(m=>(modFilter.value==='all'||(modFilter.value==='enabled'&&m.enabled)||(modFilter.value==='disabled'&&!m.enabled)||(modFilter.value==='problems'&&m.issue))&&`${m.name} ${m.modId||''} ${m.version||''}`.toLowerCase().includes(query.value.toLowerCase())));
const crumbs=computed(()=>path.value?path.value.split('/').map((name,i,parts)=>({name,path:parts.slice(0,i+1).join('/')})):[]);

const tabs=[
  {id:'mods',name:'Mods',icon:Package},
  {id:'files',name:'Files',icon:FolderOpen},
  {id:'screenshots',name:'Screenshots',icon:Monitor},
  {id:'backups',name:'Backups',icon:Globe},
  {id:'notes',name:'Notes',icon:FileText},
  {id:'logs',name:'Logs',icon:Terminal},
  {id:'settings',name:'Settings',icon:Settings},
  {id:'trash',name:'Trash',icon:Trash2}
];

const bytes=(n:number)=>n>=1048576?`${(n/1048576).toFixed(1)} MB`:n>=1024?`${(n/1024).toFixed(1)} KB`:`${n} B`;
const edited=(name:string)=>/\.(txt|json|json5|toml|yaml|yml|properties|log|cfg|ini|conf|md|csv|xml|mcmeta)$/i.test(name);

async function run(fn:()=>Promise<void>){if(busy.value)return;busy.value=true;error.value='';notice.value='';try{await fn()}catch(e){error.value=String(e)}finally{busy.value=false}}
async function readTab(){loading.value=true;error.value='';try{
 if(tab.value==='files')entries.value=await native('instance_files',{id:props.instance.id,path:path.value});
 if(tab.value==='mods')mods.value=await native('instance_mods',{id:props.instance.id});
 if(tab.value==='screenshots')await loadScreenshots();
 if(tab.value==='backups')backups.value=await native('instance_backups',{id:props.instance.id});
 if(tab.value==='trash')trash.value=await native('instance_trash',{id:props.instance.id});
 if(tab.value==='logs')await refreshLogs();
}catch(e){error.value=String(e)}finally{loading.value=false}}

async function refreshLogs(){try{consoleText.value=await native('instance_logs',{id:props.instance.id})}catch{consoleText.value='No game log yet. Start this instance to see its output.'}}
async function canLeave(){if(!dirty.value)return true;dialog.value='discard';return new Promise<boolean>(resolve=>{leaveAnswer=resolve})}
defineExpose({canLeave});
function dismiss(){if(leaveAnswer){leaveAnswer(false);leaveAnswer=null}error.value='';dialog.value=null}
async function selectTab(value:string){if(!await canLeave())return;editor.value=null;tab.value=value;query.value='';await readTab()}
async function back(){if(await canLeave())emit('back')}
async function openFolder(value:string){if(!await canLeave())return;editor.value=null;path.value=value;tab.value='files';query.value='';await readTab()}
async function openFile(entry:any){if(entry.directory){await openFolder(entry.path);return}if(!edited(entry.name)){selected.value=entry;dialog.value='details';return}await run(async()=>{if(!await canLeave())return;editor.value=await native('instance_text',{id:props.instance.id,path:entry.path});text.value=editor.value.content})}
async function save(){await run(async()=>{const result=await native('instance_file_action',{id:props.instance.id,action:'save',path:editor.value.path,content:text.value,expectedSha256:editor.value.sha256});editor.value={...editor.value,content:text.value,sha256:result.sha256};notice.value='Saved on this device.'})}
function formatJson(){try{text.value=JSON.stringify(JSON.parse(text.value),null,2)+'\n';error.value=''}catch{error.value='This JSON has a syntax error. Fix it before formatting.'}}
function ask(kind:string,entry?:any){error.value='';selected.value=entry||null;dialogName.value=kind==='rename'?entry.name:kind==='new-backup'?`${props.instance.name} backup`:'';dialog.value=kind}

async function confirm(){
 if(dialog.value==='discard'){editor.value=null;dialog.value=null;leaveAnswer?.(true);leaveAnswer=null;return}
 await run(async()=>{const action=dialog.value;const id=props.instance.id;
  if(action==='new-folder'||action==='new-file'){
   const name=dialogName.value.trim();if(!name||name.includes('/')||name.includes('\\'))throw Error('Enter a single file or folder name.');const target=path.value?`${path.value}/${name}`:name;
   await native('instance_file_action',{id,action:action==='new-folder'?'mkdir':'save',path:target,content:action==='new-file'&&/\.(json|mcmeta)$/i.test(name)?'{}\n':''});
  }
  if(action==='rename'){const name=dialogName.value.trim();if(!name||name.includes('/')||name.includes('\\'))throw Error('Enter a single name.');const parent=selected.value.path.split('/').slice(0,-1).join('/');await native('instance_file_action',{id,action:'rename',path:selected.value.path,target:parent?`${parent}/${name}`:name})}
  if(action==='trash')await native('instance_file_action',{id,action:'trash',path:selected.value.path});
  if(action==='new-backup'){const name=dialogName.value.trim();if(!name)throw Error('Enter a backup name.');await native('instance_backup_action',{id,action:'create',name})}
  if(action==='restore-backup')await native('instance_backup_action',{id,action:'restore',backupId:selected.value.id});
  if(action==='delete-backup')await native('instance_backup_action',{id,action:'delete',backupId:selected.value.id});
  dialog.value=null;await readTab();notice.value=action==='trash'?'Moved to Trash. You can restore it later.':action==='new-backup'?'Backup saved on this device.':action==='restore-backup'?'Backup restored to this instance.':action==='delete-backup'?'Backup deleted.':'Done.';
 })
}

async function importModFile(event:Event){
 const input=event.target as HTMLInputElement;const file=input.files?.[0];input.value='';if(!file)return;
 await run(async()=>{
  if(!file.name.toLowerCase().endsWith('.jar'))throw Error('Choose a valid .jar mod file.');
  if(!file.size||file.size>128*1024*1024)throw Error('Choose a .jar mod file up to 128 MiB.');
  const buf=new Uint8Array(await file.arrayBuffer());let binary='';const chunk=0x8000;
  for(let i=0;i<buf.length;i+=chunk)binary+=String.fromCharCode(...buf.subarray(i,i+chunk));
  await native('import_instance_mod',{id:props.instance.id,fileName:file.name,bytesBase64:btoa(binary)});
  await readTab();notice.value=`Imported ${file.name}.`;
 });
}

async function restore(entry:any){await run(async()=>{await native('instance_file_action',{id:props.instance.id,action:'restore',path:entry.id});await readTab();notice.value='Restored to its original folder.'})}
async function toggle(mod:any){await run(async()=>{await native('toggle_instance_mod',{id:props.instance.id,path:mod.path,enabled:!mod.enabled});await readTab()})}
async function toggleAllMods(enable:boolean){
  const targetMods=mods.value.filter(m=>m.enabled!==enable);
  if(!targetMods.length)return;
  await run(async()=>{
    for(const m of targetMods){
      await native('toggle_instance_mod',{id:props.instance.id,path:m.path,enabled:enable});
    }
    await readTab();
    notice.value=enable?`Enabled ${targetMods.length} mods.`:`Disabled ${targetMods.length} mods.`;
  });
}

// Instance Notes state
const instanceNotes=ref('');
const notesSavedNotice=ref(false);
let notesSaveTimer:ReturnType<typeof setTimeout>|null=null;
function loadInstanceNotes(){
  try{
    instanceNotes.value=localStorage.getItem(`solara:notes:${props.instance.id}`)||'';
  }catch{
    instanceNotes.value='';
  }
}
function saveInstanceNotes(manual=false){
  try{
    localStorage.setItem(`solara:notes:${props.instance.id}`,instanceNotes.value);
    notesSavedNotice.value=true;
    if(manual)notice.value='Instance notebook saved.';
    setTimeout(()=>{notesSavedNotice.value=false;},2500);
  }catch{
    error.value='Failed to save note locally.';
  }
}
function handleNotesInput(){
  if(notesSaveTimer)clearTimeout(notesSaveTimer);
  notesSaveTimer=setTimeout(()=>{saveInstanceNotes(false);},800);
}
function insertSnippet(type:'coords'|'server'|'todo'){
  let snippet='';
  if(type==='coords'){
    snippet=`\n📍 Coordinates:\n- Base: X: 0, Y: 70, Z: 0\n- Nether Portal: X: 0, Y: 60, Z: 0\n- Stronghold: X: 1200, Y: 30, Z: -800\n- Ancient City: X: -600, Y: -51, Z: 450\n`;
  }else if(type==='server'){
    snippet=`\n🌐 Server Details:\n- Address: mc.solara-inc.in\n- Port: 25565\n- Version: Minecraft ${props.instance.gameVersion}\n`;
  }else if(type==='todo'){
    snippet=`\n📋 To-Do List:\n- [ ] Gather Netherite upgrade template\n- [ ] Build automated iron farm\n- [ ] Set up villager trading hall\n- [ ] Brew fire resistance potions\n`;
  }
  instanceNotes.value=(instanceNotes.value?instanceNotes.value+'\n':'')+snippet;
  saveInstanceNotes(false);
}
const noteStats=computed(()=>{
  const t=instanceNotes.value;
  const chars=t.length;
  const words=t.trim()?t.trim().split(/\s+/).length:0;
  const lines=t?t.split('\n').length:0;
  return{chars,words,lines};
});

// Window Display Preferences & Custom JVM Arguments
const displayPrefs=ref({
  resolution:'default',
  fullscreen:false,
  customWidth:1920,
  customHeight:1080,
  jvmPreset:'default',
  jvmCustomArgs:''
});
function loadDisplayPrefs(){
  try{
    const raw=localStorage.getItem(`solara:display:${props.instance.id}`);
    if(raw)Object.assign(displayPrefs.value,JSON.parse(raw));
  }catch{}
}
function saveDisplayPrefs(){
  try{
    localStorage.setItem(`solara:display:${props.instance.id}`,JSON.stringify(displayPrefs.value));
  }catch{}
}

// Screenshots Gallery & Lightbox Viewer
interface ScreenshotItem {
  name: string;
  path: string;
  size: number;
  modifiedAt?: number;
  dataUrl?: string;
}
const screenshots = ref<ScreenshotItem[]>([]);
const screenshotQuery = ref('');
const activeLightbox = ref<ScreenshotItem | null>(null);

const visibleScreenshots = computed(() => {
  if (!screenshotQuery.value.trim()) return screenshots.value;
  const q = screenshotQuery.value.trim().toLowerCase();
  return screenshots.value.filter(s => s.name.toLowerCase().includes(q));
});

const lightboxIndex = computed(() => {
  if (!activeLightbox.value) return -1;
  return visibleScreenshots.value.findIndex(s => s.path === activeLightbox.value?.path);
});

async function loadScreenshots() {
  try {
    const raw = await native('instance_files', { id: props.instance.id, path: 'screenshots' });
    if (Array.isArray(raw)) {
      const filtered: ScreenshotItem[] = raw
        .filter((f: any) => !f.directory && /\.(png|jpg|jpeg|webp)$/i.test(f.name))
        .map((f: any) => ({
          name: f.name,
          path: f.path,
          size: f.size,
          modifiedAt: f.modifiedAt || f.modified_at
        }))
        .sort((a, b) => (b.modifiedAt || 0) - (a.modifiedAt || 0));
      screenshots.value = filtered;
      // Preload thumbnails for instant display
      for (const item of filtered.slice(0, 30)) {
        if (!item.dataUrl) {
          native('instance_image', { id: props.instance.id, path: item.path })
            .then(url => { item.dataUrl = url; })
            .catch(() => {});
        }
      }
    } else {
      screenshots.value = [];
    }
  } catch {
    screenshots.value = [];
  }
}

async function openLightbox(item: ScreenshotItem) {
  if (!item.dataUrl) {
    try {
      item.dataUrl = await native('instance_image', { id: props.instance.id, path: item.path });
    } catch {}
  }
  activeLightbox.value = item;
}

function closeLightbox() {
  activeLightbox.value = null;
}

function prevScreenshot() {
  const idx = lightboxIndex.value;
  if (idx > 0) {
    openLightbox(visibleScreenshots.value[idx - 1]);
  }
}

function nextScreenshot() {
  const idx = lightboxIndex.value;
  if (idx >= 0 && idx < visibleScreenshots.value.length - 1) {
    openLightbox(visibleScreenshots.value[idx + 1]);
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (activeLightbox.value) {
    if (e.key === 'Escape') {
      closeLightbox();
      e.preventDefault();
    } else if (e.key === 'ArrowLeft') {
      prevScreenshot();
      e.preventDefault();
    } else if (e.key === 'ArrowRight') {
      nextScreenshot();
      e.preventDefault();
    }
  }
}
if (typeof window !== 'undefined') {
  window.addEventListener('keydown', handleKeydown);
}

// Memory Performance Gauge
const ramCategory = computed(() => {
  const ram = form.value.memory;
  if (ram <= 3072) {
    return {
      label: 'Lightweight & Vanilla',
      desc: 'Optimal for Vanilla Minecraft or small mod setups (up to 30 mods). Keeps system RAM completely free.',
      color: '#38bdf8',
      score: 'Light'
    };
  } else if (ram <= 6144) {
    return {
      label: 'Standard Modded (Sweet Spot)',
      desc: 'Recommended for modern modpacks (50-150 mods). Provides stable FPS and garbage collection.',
      color: '#0ea5e9',
      score: 'Optimal'
    };
  } else if (ram <= 12288) {
    return {
      label: 'Heavy Modpacks & Shaders',
      desc: 'Built for huge community modpacks (200+ mods) and high-resolution shader packs.',
      color: '#38bdf8',
      score: 'High Power'
    };
  } else {
    return {
      label: 'Extreme Workload',
      desc: 'Large memory assignment. Ensure Windows has at least 4-8 GiB free RAM remaining for OS tasks.',
      color: '#fbbf24',
      score: 'Heavy'
    };
  }
});

// Modpack Exporter State
const exportModalOpen = ref(false);
const exportName = ref('');
const exporting = ref(false);
const exportDoneNotice = ref('');

function openExportModal() {
  exportName.value = `${props.instance.name} modpack`;
  exportDoneNotice.value = '';
  exportModalOpen.value = true;
}

async function executeExport() {
  if (!exportName.value.trim()) return;
  exporting.value = true;
  exportDoneNotice.value = '';
  try {
    await native('instance_backup_action', {
      id: props.instance.id,
      action: 'create',
      name: exportName.value.trim()
    });
    exportDoneNotice.value = 'Modpack package generated successfully and saved in instance backups.';
    notice.value = 'Modpack export ready in Backups.';
    await readTab();
  } catch (e) {
    error.value = String(e);
  } finally {
    exporting.value = false;
  }
}

async function saveSettings(){await run(async()=>{await native('edit_instance',{id:props.instance.id,...form.value});saveDisplayPrefs();notice.value='Instance settings saved.';emit('refresh')})}
async function copy(value:string){try{await navigator.clipboard.writeText(value);notice.value='Copied.'}catch{error.value='Clipboard is unavailable. Select the text and copy it manually.'}}
async function cancelTask(){try{await native('download_action',{id:task.value.id,action:'cancel'});notice.value='Cancelling…';emit('refresh')}catch(e){error.value=String(e)}}

watch(()=>props.instance.id,()=>{tab.value='mods';path.value='';entries.value=[];mods.value=[];screenshots.value=[];screenshotQuery.value='';activeLightbox.value=null;backups.value=[];trash.value=[];consoleText.value='';query.value='';editor.value=null;error.value='';notice.value='';form.value={name:props.instance.name,memory:props.instance.memory||4096,javaPath:props.instance.javaPath||'auto'};loadInstanceNotes();loadDisplayPrefs();},{immediate:true});
watch(dialog,async value=>{await nextTick();if(value&&!dialogElement.value?.open)dialogElement.value?.showModal();if(!value&&dialogElement.value?.open)dialogElement.value.close()});
watch(()=>[tab.value,props.instance.running,liveLogs.value],()=>{if(logTimer)clearInterval(logTimer);logTimer=null;if(tab.value==='logs'&&props.instance.running&&liveLogs.value)logTimer=setInterval(()=>void refreshLogs(),2000)});
watch(visibleLogLines,async()=>{if(autoScroll.value&&tab.value==='logs'){await nextTick();if(consoleContainer.value)consoleContainer.value.scrollTop=consoleContainer.value.scrollHeight}});
onUnmounted(()=>{if(typeof window!=='undefined')window.removeEventListener('keydown',handleKeydown);if(logTimer)clearInterval(logTimer);leaveAnswer?.(false)});

const scenes=['forest-valley','alpine-lake','coastal-hamlet','canyon-settlement','moss-cave','workshop'];
function instanceScene(name?:string):string{if(!name)return scenes[0];let hash=0;for(let i=0;i<name.length;i++){hash=(hash<<5)-hash+name.charCodeAt(i);hash|=0}return scenes[Math.abs(hash)%scenes.length]}
function instanceArtSrc(name?:string):string{return `/art/${instanceScene(name)}-v1.webp`}
</script>

<template>
<section class="instance-workspace">
 <button class="workspace-back" @click="back"><ArrowLeft :size="16"/> Your library</button>

 <!-- Modern Instance Header -->
 <header class="workspace-heading solara-instance-header">
   <div class="workspace-heading-bg">
     <img :src="instanceArtSrc(instance.name)" alt="" loading="eager">
     <div class="workspace-heading-overlay"></div>
   </div>
   <div class="instance-loader-badge" :class="instance.loader">
     <div class="instance-3d-cube">⚡</div>
   </div>
   <div class="workspace-identity">
     <div class="instance-top-tagline">
       <span class="workspace-kicker">INSTALLED INSTANCE</span>
       <span class="instance-status-pill" :class="{running: instance.running}">{{instance.running ? 'Running' : instance.state}}</span>
     </div>
     <h1>{{instance.name}}</h1>
     <div class="workspace-tags">
       <span class="tag-version">Minecraft {{instance.gameVersion}}</span>
       <span class="tag-loader">{{instance.loader}}{{instance.loaderVersion?' '+instance.loaderVersion:''}}</span>
       <span class="tag-meta">⚡ Solara Optimized</span>
     </div>
   </div>
   <div class="workspace-primary-actions">
     <button class="circle-action-btn" title="Open instance folder" @click="run(async()=>{await native('open_instance_folder',{id:instance.id})})">
       <FolderOpen :size="17"/>
     </button>
     <button class="circle-action-btn" title="Screenshots gallery" @click="selectTab('screenshots')">
       <Monitor :size="17"/>
     </button>
     <button class="circle-action-btn" title="Duplicate instance" @click="emit('duplicate',instance)">
       <Copy :size="17"/>
     </button>
     <button class="circle-action-btn" title="Export modpack archive" @click="openExportModal">
       <Download :size="17"/>
     </button>
     <button class="circle-action-btn" title="Instance settings" @click="selectTab('settings')">
       <Settings :size="17"/>
     </button>
     <button v-if="instance.running" class="workspace-play stop solara-play-btn" :disabled="busy||parentBusy" @click="emit('stop',instance)">
       <Square :size="16"/> Stop game
     </button>
     <button v-else-if="!accountReady" class="workspace-play solara-play-btn gate-signin" :disabled="busy||parentBusy||!!task" @click="emit('account')">
       <Plus :size="16"/> Add Minecraft account
     </button>
     <button v-else class="workspace-play solara-play-btn" :disabled="busy||parentBusy||!!task||instance.state!=='installed'" @click="emit('play',instance)">
       <Play :size="16"/> Play
     </button>
   </div>
 </header>

 <div v-if="task" class="workspace-task">
   <Coffee/>
   <div>
     <strong>{{task.detail||'Preparing your instance…'}}</strong>
     <span v-if="task.total">{{bytes(task.downloaded)}} / {{bytes(task.total)}}</span>
     <progress v-if="task.total" :value="task.downloaded" :max="task.total"/>
     <div v-else class="workspace-indeterminate"/>
   </div>
   <button @click="cancelTask">Cancel</button>
 </div>

 <!-- Modern Navigation Tabs -->
 <nav class="workspace-tabs solara-tabs" aria-label="Instance pages">
   <button v-for="item in tabs" :key="item.id" :class="{active:tab===item.id}" :aria-current="tab===item.id?'page':undefined" @click="selectTab(item.id)">
     <component :is="item.icon" :size="16"/>
     <span>{{item.name}}</span>
     <span v-if="item.id==='mods'&&mods.length" class="tab-badge">{{mods.length}}</span>
     <span v-if="item.id==='screenshots'&&screenshots.length" class="tab-badge">{{screenshots.length}}</span>
     <span v-if="item.id==='backups'&&backups.length" class="tab-badge">{{backups.length}}</span>
   </button>
 </nav>

 <p v-if="error" class="workspace-message problem" role="alert">{{error}}</p>
 <p v-if="notice" class="workspace-message" role="status">{{notice}}</p>

 <!-- Content (Mods & Packs) Tab -->
 <template v-if="tab==='mods'">
   <div class="solara-content-toolbar">
     <div class="content-search-wrap">
       <Search :size="16"/>
       <input v-model="query" :placeholder="'Search ' + (mods.length ? mods.length : '') + ' projects…'" aria-label="Search installed mods">
     </div>
     <div class="content-action-group">
       <input ref="modInput" type="file" accept=".jar,application/java-archive" aria-label="Import mod .jar file" hidden @change="importModFile">
       <button class="solara-btn-secondary" :disabled="locked||instance.loader==='vanilla'" aria-label="Add .jar file" @click="modInput?.click()">
         <Plus :size="14"/> Add .jar file
       </button>
       <button class="solara-btn-primary" @click="emit('browse-mods')">
         <Compass :size="14"/> Browse content
       </button>
     </div>
   </div>

   <div class="solara-filter-row">
     <div class="mod-filter-pills">
       <button type="button" class="filter-pill" :class="{active: modFilter==='all'}" @click="modFilter='all'">All</button>
       <button type="button" class="filter-pill" :class="{active: modFilter==='enabled'}" @click="modFilter='enabled'">Mods</button>
       <button type="button" class="filter-pill" :class="{active: modFilter==='disabled'}" @click="modFilter='disabled'">Disabled</button>
       <button type="button" class="filter-pill" :class="{active: modFilter==='problems'}" @click="modFilter='problems'">Problems</button>
     </div>
     <div class="mod-row-tools">
       <button v-if="mods.length" type="button" class="link-action-btn" :disabled="locked||!mods.some(m=>!m.enabled)" title="Enable all mods" @click="toggleAllMods(true)">
         <Check :size="13"/> Enable all
       </button>
       <button v-if="mods.length" type="button" class="link-action-btn" :disabled="locked||!mods.some(m=>m.enabled)" title="Disable all mods" @click="toggleAllMods(false)">
         <X :size="13"/> Disable all
       </button>
       <button type="button" class="link-action-btn" title="Open mods folder in system explorer" @click="run(async()=>{await native('open_instance_folder',{id:instance.id,path:'mods'})})">
         <FolderOpen :size="13"/> Mods folder
       </button>
       <button class="link-action-btn" title="Refresh mods" :disabled="loading" @click="readTab">
         <RefreshCw :size="13"/> Refresh
       </button>
     </div>
   </div>

   <p v-if="instance.loader==='vanilla'" class="workspace-message">Vanilla does not load mods. Use a Fabric, Quilt or Forge instance to add community mods.</p>

   <div v-if="loading" class="workspace-empty">Reading mod files…</div>
   <div v-else-if="!visibleMods.length" class="workspace-empty">
     <Package/>
     <h3>{{mods.length?'No mods match your filter':'No content installed yet'}}</h3>
     <p>{{mods.length?'Try another search term or reset filter.':'Add mods, resource packs or shaders to customize your game.'}}</p>
     <button class="solara-primary-btn" @click="emit('browse-mods')"><Compass :size="14"/> Browse Mods & Addons</button>
   </div>
   <div v-else class="project-table">
     <div class="project-table-header">
       <span>Project</span>
       <span>Loader</span>
       <span>Size</span>
       <span style="text-align:right">Actions</span>
     </div>
     <article v-for="mod in visibleMods" :key="mod.path" class="project-table-row" :class="{disabled:!mod.enabled}">
       <div class="mod-project-details">
         <div class="mod-project-avatar">
           <Package :size="18"/>
         </div>
         <div class="mod-project-info">
           <strong>{{mod.name}}</strong>
           <span>{{mod.version||'Release'}}<template v-if="mod.issue"> · <span class="problem-text">{{mod.issue}}</span></template></span>
         </div>
       </div>
       <div class="mod-project-chips">
         <span class="loader-chip">{{mod.loaders?.join(', ')||instance.loader}}</span>
       </div>
       <span class="version-chip">{{bytes(mod.size)}}</span>
       <div style="display:flex;align-items:center;justify-content:flex-end;gap:8px">
         <button role="switch" :aria-checked="mod.enabled" :aria-label="(mod.enabled?'Disable ':'Enable ')+mod.name" :disabled="locked" class="workspace-switch" @click="toggle(mod)">
           <i/>
         </button>
         <button :aria-label="'Move '+mod.name+' to Trash'" :disabled="locked" class="workspace-icon-button" @click="ask('trash',{...mod,name:mod.fileName})">
           <Trash2 :size="15"/>
         </button>
       </div>
     </article>
   </div>
 </template>

 <!-- Files Tab -->
 <template v-if="tab==='files'">
   <div class="workspace-section-heading">
     <div><h2>Instance files</h2><p>Files inside this instance. Saves stay separate from game updates.</p></div>
     <div class="row">
       <button class="solara-secondary-btn" :disabled="locked" @click="ask('new-folder')"><Plus :size="14"/> Folder</button>
       <button class="solara-secondary-btn" :disabled="locked" @click="ask('new-file')"><FileText :size="14"/> Text file</button>
     </div>
   </div>
   <div class="workspace-tools">
     <div class="workspace-breadcrumb">
       <button @click="openFolder('')">Files</button>
       <template v-for="crumb in crumbs" :key="crumb.path">
         <ChevronRight :size="13"/>
         <button @click="openFolder(crumb.path)">{{crumb.name}}</button>
       </template>
     </div>
     <button title="Refresh files" :disabled="loading||dirty" @click="readTab"><RefreshCw :size="14"/></button>
   </div>
   <template v-if="editor">
     <div class="workspace-editor-bar">
       <span><FileText/>{{editor.path}} <em v-if="dirty">Unsaved</em></span>
       <div class="row">
         <button v-if="/\.(json|mcmeta)$/i.test(editor.path)" :disabled="locked" @click="formatJson">Format JSON</button>
         <button :disabled="locked||!dirty" class="solara-primary-btn" @click="save"><Save :size="14"/> Save</button>
         <button aria-label="Close file editor" @click="canLeave().then(ok=>{if(ok)editor=null})"><X :size="14"/></button>
       </div>
     </div>
     <textarea v-model="text" class="workspace-editor" aria-label="File contents" spellcheck="false" :readonly="locked"/>
   </template>
   <template v-else>
     <label class="workspace-search"><Search/><input v-model="query" aria-label="Search instance files" placeholder="Search this folder…"></label>
     <div v-if="loading" class="workspace-empty">Reading files…</div>
     <div v-else-if="!visibleFiles.length" class="workspace-empty">
       <FolderOpen/>
       <h3>{{query?'No matching files':'This folder is empty'}}</h3>
       <p>{{query?'Try another name.':'New game files will appear when Minecraft creates them.'}}</p>
     </div>
     <div v-else class="workspace-file-list">
       <article v-for="file in visibleFiles" :key="file.path" class="workspace-file-row">
         <button class="workspace-file-open" @click="openFile(file)">
           <span class="workspace-file-symbol"><FolderOpen v-if="file.directory"/><FileText v-else/></span>
           <span>{{file.name}}</span>
         </button>
         <span class="workspace-file-size">{{file.directory?'Folder':bytes(file.size)}}</span>
         <button :aria-label="'Rename '+file.name" :disabled="locked" class="workspace-icon-button" @click="ask('rename',file)"><Pencil/></button>
         <button :aria-label="'Move '+file.name+' to Trash'" :disabled="locked" class="workspace-icon-button" @click="ask('trash',file)"><Trash2/></button>
       </article>
     </div>
   </template>
 </template>

 <!-- Screenshots Tab -->
 <template v-if="tab==='screenshots'">
   <div class="workspace-section-heading">
     <div>
       <h2>In-game screenshots <span>{{screenshots.length}}</span></h2>
       <p>High-resolution captures taken in Minecraft with F2. Saved directly inside this instance.</p>
     </div>
     <div class="row">
       <button type="button" class="solara-secondary-btn" title="Open screenshots folder in system file explorer" @click="run(async () => { await native('open_instance_folder', { id: instance.id, path: 'screenshots' }); })">
         <FolderOpen :size="14"/> Open folder
       </button>
       <button type="button" class="solara-secondary-btn" title="Refresh screenshots" :disabled="loading" @click="readTab">
         <RefreshCw :size="14"/> Refresh
       </button>
     </div>
   </div>

   <div v-if="loading" class="workspace-empty">Reading screenshots…</div>
   <div v-else-if="!screenshots.length" class="workspace-empty">
     <Monitor :size="32"/>
     <h3>No screenshots captured yet</h3>
     <p>Press <strong>F2</strong> while playing Minecraft to take screenshots. They will automatically appear in this gallery.</p>
     <button type="button" class="solara-primary-btn" @click="run(async () => { await native('open_instance_folder', { id: instance.id, path: 'screenshots' }); })">
       <FolderOpen :size="14"/> Open screenshots folder
     </button>
   </div>
   <div v-else>
     <div class="solara-content-toolbar" style="margin-bottom:14px">
       <div class="content-search-wrap">
         <Search :size="16"/>
         <input v-model="screenshotQuery" placeholder="Filter screenshots by filename…" aria-label="Filter screenshots">
       </div>
     </div>

     <div class="screenshots-gallery-grid">
       <article v-for="item in visibleScreenshots" :key="item.path" class="screenshot-card" @click="openLightbox(item)">
         <div class="screenshot-thumb-wrap">
           <img v-if="item.dataUrl" :src="item.dataUrl" :alt="item.name" loading="lazy">
           <div v-else class="screenshot-thumb-loading">
             <Monitor :size="24"/>
             <span>Loading photo…</span>
           </div>
           <div class="screenshot-hover-overlay">
             <span class="hover-view-text">Click to expand</span>
           </div>
         </div>
         <div class="screenshot-meta-row">
           <span class="screenshot-name" :title="item.name">{{ item.name }}</span>
           <span class="screenshot-size">{{ bytes(item.size) }}</span>
         </div>
         <div class="screenshot-sub-meta">
           <span>{{ item.modifiedAt ? new Date(item.modifiedAt * 1000).toLocaleDateString(undefined, {month:'short', day:'numeric', hour:'2-digit', minute:'2-digit'}) : '' }}</span>
           <button type="button" class="screenshot-delete-btn" title="Move to Trash" @click.stop="ask('trash', item)">
             <Trash2 :size="13"/>
           </button>
         </div>
       </article>
     </div>
   </div>
 </template>

 <!-- Worlds & Backups Tab -->
 <template v-if="tab==='backups'">
   <div class="workspace-section-heading">
     <div><h2>World &amp; config backups <span>{{backups.length}}</span></h2><p>Snapshots of your worlds and settings stored safely inside this instance.</p></div>
     <div class="row">
       <button class="solara-primary-btn" :disabled="locked" @click="ask('new-backup')"><Plus :size="14"/> Create backup</button>
       <button type="button" class="solara-secondary-btn" title="Open backups folder in system explorer" @click="run(async () => { await native('open_instance_folder', { id: instance.id, path: 'backups' }); })">
         <FolderOpen :size="14"/> Backups folder
       </button>
       <button title="Refresh backups" :disabled="loading" @click="readTab"><RefreshCw :size="14"/></button>
     </div>
   </div>
   <div v-if="loading" class="workspace-empty">Reading backups…</div>
   <div v-else-if="!backups.length" class="workspace-empty">
     <Globe/>
     <h3>No world backups yet</h3>
     <p>Save a snapshot of your worlds and config files before changing mods or updating.</p>
     <button class="solara-primary-btn" :disabled="locked" @click="ask('new-backup')"><Plus :size="14"/> Create backup</button>
   </div>
   <div v-else class="workspace-file-list">
     <article v-for="entry in backups" :key="entry.id" class="workspace-file-row">
       <span class="workspace-file-symbol"><Globe/></span>
       <div class="workspace-row-name">
         <strong>{{entry.name}}</strong>
         <span>{{entry.worldCount||0}} {{entry.worldCount===1?'world':'worlds'}} · {{entry.fileCount||0}} {{entry.fileCount===1?'file':'files'}} · {{new Date(entry.createdAt).toLocaleString()}}</span>
       </div>
       <span class="workspace-file-size">{{bytes(entry.size||0)}}</span>
       <button :aria-label="'Restore backup '+entry.name" :disabled="locked" class="solara-secondary-btn" @click="ask('restore-backup',entry)"><RotateCcw :size="14"/> Restore</button>
       <button :aria-label="'Delete backup '+entry.name" :disabled="locked" class="workspace-icon-button" @click="ask('delete-backup',entry)"><Trash2/></button>
     </article>
   </div>
 </template>

 <!-- Notes Tab -->
 <template v-if="tab==='notes'">
   <div class="workspace-section-heading">
     <div>
       <h2>Instance notebook</h2>
       <p>Keep coordinates, server addresses, modpack notes, and todos safely stored on this device.</p>
     </div>
     <div class="row">
       <button type="button" class="solara-secondary-btn" @click="insertSnippet('coords')"><Compass :size="14"/> Coordinates</button>
       <button type="button" class="solara-secondary-btn" @click="insertSnippet('todo')"><Check :size="14"/> Todo list</button>
       <button type="button" class="solara-secondary-btn" @click="insertSnippet('server')"><Globe :size="14"/> Server info</button>
       <button type="button" class="solara-secondary-btn" :disabled="!instanceNotes" @click="copy(instanceNotes)"><Copy :size="14"/> Copy</button>
       <button type="button" class="solara-primary-btn" @click="saveInstanceNotes(true)"><Save :size="14"/> Save note</button>
     </div>
   </div>

   <div class="notes-workspace-container">
     <div class="notes-meta-bar">
       <div class="notes-stats">
         <span>{{ noteStats.lines }} {{ noteStats.lines === 1 ? 'line' : 'lines' }}</span>
         <span>·</span>
         <span>{{ noteStats.words }} {{ noteStats.words === 1 ? 'word' : 'words' }}</span>
         <span>·</span>
         <span>{{ noteStats.chars }} characters</span>
       </div>
       <div class="notes-status-badge">
         <span v-if="notesSavedNotice" class="badge-saved"><Check :size="12"/> Saved to device</span>
         <span v-else class="badge-autosave">⚡ Auto-saves locally</span>
       </div>
     </div>
     <textarea
       v-model="instanceNotes"
       class="notes-textarea"
       placeholder="Write coordinates, seeds, server IPs, or modpack ideas here... Content is preserved per-instance and auto-saves automatically as you type."
       spellcheck="false"
       @input="handleNotesInput"
     />
   </div>
 </template>

 <!-- Trash Tab -->
 <template v-if="tab==='trash'">
   <div class="workspace-section-heading">
     <div><h2>Nothing lost.</h2><p>Files removed here can be restored to their original folder.</p></div>
     <button title="Refresh Trash" :disabled="loading" @click="readTab"><RefreshCw :size="14"/></button>
   </div>
   <div v-if="loading" class="workspace-empty">Reading Trash…</div>
   <div v-else-if="!trash.length" class="workspace-empty">
     <Trash2/>
     <h3>Trash is empty</h3>
     <p>Removed files will wait here until you restore them.</p>
   </div>
   <div v-else class="workspace-file-list">
     <article v-for="entry in trash" :key="entry.id" class="workspace-file-row">
       <span class="workspace-file-symbol"><FolderOpen v-if="entry.directory"/><FileText v-else/></span>
       <div class="workspace-row-name">
         <strong>{{entry.originalPath}}</strong>
         <span>{{new Date(entry.removedAt).toLocaleString()}}</span>
       </div>
       <button :disabled="locked" class="solara-secondary-btn" @click="restore(entry)"><RotateCcw :size="14"/> Restore</button>
     </article>
   </div>
 </template>

 <!-- Logs Tab -->
 <template v-if="tab==='logs'">
   <div class="workspace-section-heading">
     <div><h2>Game output</h2><p>Live game console and log records. Account credentials are automatically redacted.</p></div>
     <div class="row">
       <button class="solara-secondary-btn" @click="liveLogs=!liveLogs">{{liveLogs?'Pause live stream':'Resume live stream'}}</button>
       <button class="solara-secondary-btn" :disabled="!visibleLogLines.length" @click="copyFilteredLogs"><Copy :size="14"/> Copy logs</button>
       <button title="Refresh game logs" @click="refreshLogs"><RefreshCw :size="14"/></button>
     </div>
   </div>

   <div class="console-controls-bar">
     <div class="console-search-wrap">
       <Search :size="14"/>
       <input v-model="logSearch" placeholder="Filter log output…" aria-label="Filter log output">
       <button v-if="logSearch" type="button" class="search-clear-btn" @click="logSearch=''">
         <X :size="12"/>
       </button>
     </div>

     <div class="console-filter-pills">
       <button type="button" class="filter-pill" :class="{active: logSeverity==='all'}" @click="logSeverity='all'">
         All ({{ logCounts.total }})
       </button>
       <button type="button" class="filter-pill pill-error" :class="{active: logSeverity==='error'}" @click="logSeverity='error'">
         Errors ({{ logCounts.error }})
       </button>
       <button type="button" class="filter-pill pill-warn" :class="{active: logSeverity==='warn'}" @click="logSeverity='warn'">
         Warnings ({{ logCounts.warn }})
       </button>
       <button type="button" class="filter-pill pill-info" :class="{active: logSeverity==='info'}" @click="logSeverity='info'">
         Info ({{ logCounts.info }})
       </button>
       <button type="button" class="filter-pill pill-chat" :class="{active: logSeverity==='chat'}" @click="logSeverity='chat'">
         Chat ({{ logCounts.chat }})
       </button>
     </div>

     <div class="console-toggles">
       <button type="button" class="toggle-pill-btn" :class="{active: wrapLogs}" title="Wrap long lines" @click="wrapLogs=!wrapLogs">
         Wrap
       </button>
       <button type="button" class="toggle-pill-btn" :class="{active: autoScroll}" title="Auto-scroll to latest output" @click="autoScroll=!autoScroll">
         Scroll
       </button>
       <button type="button" class="toggle-pill-btn" title="Clear console view" @click="clearLogView">
         Clear
       </button>
     </div>
   </div>

   <div ref="consoleContainer" class="solara-console-viewport" :class="{'wrap-lines': wrapLogs}">
     <div v-if="!visibleLogLines.length" class="console-empty-state">
       <Terminal :size="24"/>
       <p>{{ logSearch || logSeverity!=='all' ? 'No log lines match your active filters.' : 'No output yet. Start this instance to see live console output.' }}</p>
     </div>
     <div v-else class="console-lines-wrapper">
       <div v-for="line in visibleLogLines" :key="line.id" class="console-line" :class="'line-'+line.level">
         <span class="line-gutter">{{ line.id + 1 }}</span>
         <span class="line-text">{{ line.text }}</span>
       </div>
     </div>
   </div>
 </template>

 <!-- Settings Tab -->
 <template v-if="tab==='settings'">
   <div class="workspace-section-heading">
     <div><h2>Instance preferences</h2><p>Settings apply to this instance only.</p></div>
   </div>
   <form class="workspace-settings" @submit.prevent="saveSettings">
     <label>Instance name<input v-model="form.name" required maxlength="80"></label>
     <label>Memory limit: <strong>{{(form.memory/1024).toFixed(1)}} GiB</strong> ({{form.memory}} MiB)
       <div class="ram-gauge-card">
         <div class="ram-gauge-graphic">
           <svg class="ram-gauge-svg" viewBox="0 0 100 100">
             <circle class="gauge-bg" cx="50" cy="50" r="40" />
             <circle
               class="gauge-progress"
               cx="50"
               cy="50"
               r="40"
               :stroke="ramCategory.color"
               :stroke-dasharray="251.2"
               :stroke-dashoffset="251.2 - (251.2 * Math.min(100, Math.max(5, (form.memory / 16384) * 100))) / 100"
             />
           </svg>
           <div class="gauge-center-content">
             <span class="gauge-score" :style="{color: ramCategory.color}">{{ ramCategory.score }}</span>
             <span class="gauge-val">{{ (form.memory/1024).toFixed(1) }}G</span>
           </div>
         </div>
         <div class="ram-gauge-info">
           <div class="ram-category-badge" :style="{borderColor: ramCategory.color, color: ramCategory.color}">
             <Activity :size="13" />
             <span>{{ ramCategory.label }}</span>
           </div>
           <p class="ram-category-desc">{{ ramCategory.desc }}</p>
         </div>
       </div>
       <div class="preset-pills" style="margin:6px 0">
         <button v-for="ram in [2048,4096,6144,8192,12288,16384]" :key="ram" type="button" class="preset-pill" :class="{active:form.memory===ram}" @click="form.memory=ram">{{ram>=1024?(ram/1024)+'G':ram+'M'}}</button>
       </div>
       <div class="ram-meter-bar">
         <div class="ram-meter-fill" :style="{width: Math.min(100, Math.max(5, (form.memory / 16384) * 100)) + '%'}"></div>
       </div>
       <div class="ram-meter-labels">
         <span>512M</span>
         <span>4G (Standard)</span>
         <span>8G (Modpacks)</span>
         <span>16G+</span>
       </div>
       <input v-model.number="form.memory" type="number" min="512" max="65536" step="512" required>
       <small>Leave enough memory for Windows and other apps.</small>
     </label>
     <label>Java executable
       <input v-model="form.javaPath" placeholder="auto" required>
       <small>Use auto for managed Java, java for PATH, or choose your own executable path.</small>
     </label>

     <!-- Display & Window Preferences -->
     <div class="settings-subgroup">
       <h3 class="settings-subgroup-title">Display &amp; Window Preferences</h3>
       <p class="settings-subgroup-hint">Configure launch window dimensions and presentation for this instance.</p>
       <label>Window size preset
         <select v-model="displayPrefs.resolution" class="solara-select" @change="saveDisplayPrefs">
           <option value="default">Default (Standard windowed)</option>
           <option value="1080p">1920 × 1080 (Full HD 1080p)</option>
           <option value="720p">1280 × 720 (HD 720p)</option>
           <option value="1440p">2560 × 1440 (QHD 1440p)</option>
           <option value="custom">Custom dimensions</option>
         </select>
       </label>
       <div v-if="displayPrefs.resolution==='custom'" class="row-inputs" style="display:flex;gap:12px">
         <label style="flex:1">Width (px)
           <input v-model.number="displayPrefs.customWidth" type="number" min="640" max="7680" step="10" @input="saveDisplayPrefs">
         </label>
         <label style="flex:1">Height (px)
           <input v-model.number="displayPrefs.customHeight" type="number" min="480" max="4320" step="10" @input="saveDisplayPrefs">
         </label>
       </div>
       <label class="checkbox-option" style="margin-top:6px;display:flex;align-items:center;gap:8px;cursor:pointer">
         <input v-model="displayPrefs.fullscreen" type="checkbox" @change="saveDisplayPrefs">
         <span>Launch Minecraft in Fullscreen mode</span>
       </label>
     </div>

     <!-- JVM & Garbage Collector Flags -->
     <div class="settings-subgroup">
       <h3 class="settings-subgroup-title">Java Virtual Machine Flags</h3>
       <p class="settings-subgroup-hint">Tune garbage collector presets for smoother frame rates and stutter reduction.</p>
       <div class="preset-pills" style="margin:8px 0">
         <button type="button" class="preset-pill" :class="{active:displayPrefs.jvmPreset==='default'}" @click="displayPrefs.jvmPreset='default';saveDisplayPrefs()">Solara Default</button>
         <button type="button" class="preset-pill" :class="{active:displayPrefs.jvmPreset==='g1gc'}" @click="displayPrefs.jvmPreset='g1gc';saveDisplayPrefs()">G1GC Balanced</button>
         <button type="button" class="preset-pill" :class="{active:displayPrefs.jvmPreset==='zgc'}" @click="displayPrefs.jvmPreset='zgc';saveDisplayPrefs()">Low-Latency ZGC</button>
         <button type="button" class="preset-pill" :class="{active:displayPrefs.jvmPreset==='custom'}" @click="displayPrefs.jvmPreset='custom';saveDisplayPrefs()">Custom</button>
       </div>
       <label v-if="displayPrefs.jvmPreset==='custom'">Custom JVM Arguments
         <input v-model="displayPrefs.jvmCustomArgs" placeholder="-XX:+UseG1GC -XX:G1ReservePercent=20" @input="saveDisplayPrefs">
       </label>
     </div>
     <button class="workspace-play solara-primary-btn" :disabled="locked"><Save :size="14"/> Save changes</button>
     <p v-if="instance.running" class="workspace-message">Close Minecraft before changing files or settings.</p>
   </form>
 </template>

 <dialog ref="dialogElement" class="workspace-dialog" aria-labelledby="workspace-dialog-title" @cancel.prevent="dismiss">
   <form @submit.prevent="confirm">
     <header>
       <h2 id="workspace-dialog-title">{{({'new-folder':'New folder','new-file':'New text file','new-backup':'Create backup','restore-backup':'Restore backup?','delete-backup':'Delete backup?',rename:'Rename',trash:'Move to Trash?',discard:'Unsaved changes',details:'File details'} as Record<string,string>)[dialog||'']}}</h2>
       <button type="button" aria-label="Close instance dialog" @click="dismiss"><X :size="16"/></button>
     </header>
     <template v-if="dialog==='new-folder'||dialog==='new-file'||dialog==='rename'||dialog==='new-backup'">
       <label>Name<input v-model="dialogName" autofocus required :maxlength="dialog==='new-backup'?80:180"></label>
       <p v-if="dialog==='new-file'">Use a text type such as .txt, .json, .toml or .properties.</p>
       <p v-if="dialog==='new-backup'">Saves a verified snapshot of this instance's worlds and config files.</p>
     </template>
     <p v-if="dialog==='trash'">{{selected?.name}} will move to this instance's Trash. You can restore it later.</p>
     <p v-if="dialog==='restore-backup'">Restore {{selected?.name}}? Saved worlds and config files from this snapshot will be restored to this instance.</p>
     <p v-if="dialog==='delete-backup'">Delete backup {{selected?.name}} permanently from this instance?</p>
     <p v-if="dialog==='discard'">This file has unsaved changes. Leave without saving?</p>
     <dl v-if="dialog==='details'" class="workspace-facts">
       <div><dt>Path</dt><dd>{{selected?.path}}</dd></div>
       <div><dt>Size</dt><dd>{{bytes(selected?.size||0)}}</dd></div>
       <div><dt>Modified</dt><dd>{{selected?.modifiedAt?new Date(selected.modifiedAt*1000).toLocaleString():'Unknown'}}</dd></div>
     </dl>
     <p v-if="error" role="alert" class="workspace-message problem">{{error}}</p>
     <footer>
       <button type="button" @click="dismiss">{{dialog==='details'?'Close':'Cancel'}}</button>
       <button v-if="dialog!=='details'" :disabled="busy" class="workspace-play" :class="{danger:dialog==='trash'||dialog==='delete-backup'}">
         {{dialog==='trash'?'Move to Trash':dialog==='delete-backup'?'Delete backup':dialog==='restore-backup'?'Restore backup':dialog==='new-backup'?'Create backup':dialog==='discard'?'Discard changes':'Save'}}
       </button>
     </footer>
   </form>
 </dialog>

 <!-- Screenshot Lightbox Modal -->
 <dialog v-if="activeLightbox" class="solara-lightbox-dialog" open @click.self="closeLightbox">
   <div class="lightbox-header">
     <div class="lightbox-title-wrap">
       <strong class="lightbox-filename">{{ activeLightbox.name }}</strong>
       <span class="lightbox-meta">{{ bytes(activeLightbox.size) }} · {{ activeLightbox.modifiedAt ? new Date(activeLightbox.modifiedAt * 1000).toLocaleString() : '' }} · {{ lightboxIndex + 1 }} of {{ visibleScreenshots.length }}</span>
     </div>
     <div class="lightbox-actions">
       <button type="button" class="solara-secondary-btn" title="Open in system explorer" @click="run(async () => { await native('open_instance_folder', { id: instance.id, path: activeLightbox?.path }); })">
         <FolderOpen :size="14"/> Show in folder
       </button>
       <button type="button" class="circle-action-btn" title="Close" @click="closeLightbox">
         <X :size="16"/>
       </button>
     </div>
   </div>

   <div class="lightbox-viewport">
     <button v-if="lightboxIndex > 0" type="button" class="lightbox-nav-btn prev" aria-label="Previous screenshot" @click.stop="prevScreenshot">
       <ChevronLeft :size="24"/>
     </button>
     <div class="lightbox-img-wrapper">
       <img v-if="activeLightbox.dataUrl" :src="activeLightbox.dataUrl" :alt="activeLightbox.name">
       <div v-else class="lightbox-loading">
         <Monitor :size="32"/>
         <span>Loading high-resolution screenshot…</span>
       </div>
     </div>
     <button v-if="lightboxIndex < visibleScreenshots.length - 1" type="button" class="lightbox-nav-btn next" aria-label="Next screenshot" @click.stop="nextScreenshot">
       <ChevronRight :size="24"/>
     </button>
   </div>
 </dialog>

  <!-- Modpack Exporter Modal -->
  <dialog v-if="exportModalOpen" class="solara-lightbox-dialog" open @click.self="exportModalOpen = false">
    <div class="lightbox-header" style="max-width: 520px; width: 100%;">
      <div class="lightbox-title-wrap">
        <strong class="lightbox-filename" style="display:flex;align-items:center;gap:8px">
          <Download :size="16" style="color:#38bdf8"/> Export Modpack Snapshot
        </strong>
        <span class="lightbox-meta">Bundle worlds, configs, and mods into a portable backup archive</span>
      </div>
      <button type="button" class="circle-action-btn" title="Close" @click="exportModalOpen = false">
        <X :size="16"/>
      </button>
    </div>

    <div class="export-modal-body" style="max-width: 520px; width: 100%; background: #0b111e; border: 1px solid rgba(56,189,248,0.2); border-radius: 12px; padding: 20px; margin-top: 12px; display: flex; flex-direction: column; gap: 16px;">
      <p style="color: #94a3b8; font-size: 13px; line-height: 1.5; margin: 0;">
        Create an independent, packaged archive of <strong>{{ instance.name }}</strong>. You can restore this archive anytime or transfer it to another Solara installation.
      </p>

      <label style="display: flex; flex-direction: column; gap: 6px; color: #cbd5e1; font-size: 13px; font-weight: 500;">
        Archive Name
        <input v-model="exportName" class="solara-input" placeholder="e.g. My Modpack v1.0" maxlength="80" style="background: rgba(15,23,42,0.8); border: 1px solid rgba(56,189,248,0.3); border-radius: 8px; color: #f1f5f9; padding: 10px 14px; font-size: 14px; outline: none;">
      </label>

      <div v-if="exportDoneNotice" class="export-success-badge" style="background: rgba(34,197,94,0.1); border: 1px solid rgba(34,197,94,0.3); border-radius: 8px; padding: 12px; display: flex; align-items: center; gap: 10px; color: #4ade80; font-size: 13px;">
        <Check :size="16" style="flex-shrink:0"/>
        <span>{{ exportDoneNotice }}</span>
      </div>

      <div style="display: flex; justify-content: flex-end; gap: 10px; margin-top: 8px;">
        <button type="button" class="solara-secondary-btn" @click="exportModalOpen = false">Close</button>
        <button type="button" class="solara-primary-btn" :disabled="exporting || !exportName.trim()" @click="executeExport">
          <Download :size="14" />
          <span>{{ exporting ? 'Packaging modpack...' : 'Export Snapshot' }}</span>
        </button>
      </div>
    </div>
  </dialog>
</section>
</template>
