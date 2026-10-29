import { ref, computed } from 'vue';
import { PublicCache, chooseEvent, latestRelease } from './domain.mjs';
import catalogue from './events.json';
import {features} from './features';
export const isDesktop = typeof window!=='undefined' && '__TAURI_INTERNALS__' in window;
export const snapshot=ref<any>({data:null,at:null,state:'checking'});
export const preferences=ref({theme:localStorage.getItem('solara:theme')||'auto',motion:localStorage.getItem('solara:motion')==='true',accent:/^#[a-f0-9]{6}$/i.test(localStorage.getItem('solara:accent')||'')?localStorage.getItem('solara:accent')!:'#b9a8ff'});
export const preview=ref<string|null>(null);
const developmentApi=import.meta.env.DEV?new URLSearchParams(location.search).get('api'):null;
const origin=(developmentApi&&/^http:\/\/(127\.0\.0\.1|localhost):\d+$/.test(developmentApi)?developmentApi:null)||import.meta.env.VITE_SOLARA_API_URL||'https://api.solara-inc.in';
const cache=new PublicCache({storage:localStorage,origin});
export const eventOptions=catalogue.events;
export const currentEvent=computed(()=>chooseEvent(snapshot.value.data?.events||[],{mode:features.seasons?preferences.value.theme:'neutral',preview:features.seasons?preview.value:null,override:features.seasons?snapshot.value.data?.eventOverride:null}));
export const latest=computed(()=>latestRelease(snapshot.value.data?.releases||[]));
export const publicLoading=ref(false);
export async function refreshPublic() {if(publicLoading.value)return;publicLoading.value=true;try{snapshot.value=await cache.refresh()}finally{publicLoading.value=false}}
export function savePreferences(){localStorage.setItem('solara:accent',preferences.value.accent);localStorage.setItem('solara:theme',preferences.value.theme);localStorage.setItem('solara:motion',String(preferences.value.motion));}
export async function native<T=any>(command:string,args:Record<string,unknown>={}) : Promise<T> {
  if(!isDesktop) throw new Error('This action needs the installed Solara desktop app.');
  const {invoke}=await import('@tauri-apps/api/core');return invoke<T>(command,args);
}
export async function openExternal(url:string){
  const u=new URL(url);if(u.protocol!=='https:')throw new Error('Only HTTPS destinations are allowed');
  if(isDesktop){await native('open_external',{url:u.href});}else window.open(u.href,'_blank','noopener,noreferrer');
}
export const links={home:'https://www.solara-inc.in',docs:'https://auth.solara-inc.in/docs.php',support:'https://www.solara-inc.in/contact',github:import.meta.env.VITE_SOLARA_GITHUB_URL||'https://github.com/Solara-Inc-Development/solara-launcher'};
