<script setup lang="ts">
import {ref,computed,watch,nextTick,onUnmounted} from 'vue';
import {native} from '../../packages/shared/client';
import {ArrowLeft,Box,Play,Square,FolderOpen,FileText,Trash2,Pencil,Save,RefreshCw,Search,Plus,X,Terminal,Settings,Package,RotateCcw,Coffee,Copy,ChevronRight,ShieldCheck,Globe,MoreVertical,ExternalLink,Compass,Check} from '../../packages/ui/icons';

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
  {id:'backups',name:'Backups',icon:Globe},
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
async function saveSettings(){await run(async()=>{await native('edit_instance',{id:props.instance.id,...form.value});notice.value='Instance settings saved.';emit('refresh')})}
async function copy(value:string){try{await navigator.clipboard.writeText(value);notice.value='Copied.'}catch{error.value='Clipboard is unavailable. Select the text and copy it manually.'}}
async function cancelTask(){try{await native('download_action',{id:task.value.id,action:'cancel'});notice.value='Cancelling…';emit('refresh')}catch(e){error.value=String(e)}}

watch(()=>props.instance.id,()=>{tab.value='mods';path.value='';entries.value=[];mods.value=[];backups.value=[];trash.value=[];consoleText.value='';query.value='';editor.value=null;error.value='';notice.value='';form.value={name:props.instance.name,memory:props.instance.memory||4096,javaPath:props.instance.javaPath||'auto'}},{immediate:true});
watch(dialog,async value=>{await nextTick();if(value&&!dialogElement.value?.open)dialogElement.value?.showModal();if(!value&&dialogElement.value?.open)dialogElement.value.close()});
watch(()=>[tab.value,props.instance.running,liveLogs.value],()=>{if(logTimer)clearInterval(logTimer);logTimer=null;if(tab.value==='logs'&&props.instance.running&&liveLogs.value)logTimer=setInterval(()=>void refreshLogs(),2000)});
watch(visibleLogLines,async()=>{if(autoScroll.value&&tab.value==='logs'){await nextTick();if(consoleContainer.value)consoleContainer.value.scrollTop=consoleContainer.value.scrollHeight}});
onUnmounted(()=>{if(logTimer)clearInterval(logTimer);leaveAnswer?.(false)});

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
     <button class="circle-action-btn" title="Duplicate instance" @click="emit('duplicate',instance)">
       <Copy :size="17"/>
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

 <!-- Worlds & Backups Tab -->
 <template v-if="tab==='backups'">
   <div class="workspace-section-heading">
     <div><h2>World &amp; config backups <span>{{backups.length}}</span></h2><p>Snapshots of your worlds and settings stored safely inside this instance.</p></div>
     <div class="row">
       <button class="solara-primary-btn" :disabled="locked" @click="ask('new-backup')"><Plus :size="14"/> Create backup</button>
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
       <div class="preset-pills" style="margin:6px 0">
         <button v-for="ram in [2048,4096,6144,8192,12288]" :key="ram" type="button" class="preset-pill" :class="{active:form.memory===ram}" @click="form.memory=ram">{{ram>=1024?(ram/1024)+'G':ram+'M'}}</button>
       </div>
       <input v-model.number="form.memory" type="number" min="512" max="65536" step="512" required>
       <small>Leave enough memory for Windows and other apps.</small>
     </label>
     <label>Java executable
       <input v-model="form.javaPath" placeholder="auto" required>
       <small>Use auto for managed Java, java for PATH, or choose your own executable path.</small>
     </label>
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
</section>
</template>
