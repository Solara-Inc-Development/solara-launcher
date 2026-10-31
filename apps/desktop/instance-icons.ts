import {ref} from 'vue';

const storageKey='solara:instance-icons:v1';
const raster=/^data:image\/(?:png|jpeg|webp);base64,[A-Za-z0-9+/=]+$/;
function read():Record<string,string>{
  try {
    const stored=JSON.parse(localStorage.getItem(storageKey)||'{}');
    return Object.fromEntries(Object.entries(stored).filter((entry):entry is [string,string]=>typeof entry[1]==='string'&&entry[1].length<350000&&raster.test(entry[1])));
  } catch {return {}}
}
export const instanceIcons=ref(read());
export function resetInstanceIcon(id:string){
  const next={...instanceIcons.value};delete next[id];
  localStorage.setItem(storageKey,JSON.stringify(next));instanceIcons.value=next;
}
export async function uploadInstanceIcon(id:string,file:File){
  if(!['image/png','image/jpeg','image/webp'].includes(file.type))throw Error('Choose a PNG, JPG or WebP image.');
  if(file.size>5*1024*1024)throw Error('Choose an image smaller than 5 MB.');
  const url=await new Promise<string>((resolve,reject)=>{
    const reader=new FileReader();reader.onload=()=>resolve(String(reader.result));
    reader.onerror=()=>reject(Error('Could not read this image.'));reader.readAsDataURL(file);
  });
  {
    const image=new Image();image.src=url;
    try {await image.decode()}catch {throw Error('This image could not be opened. Try another PNG, JPG or WebP.')}
    if(image.width*image.height>16777216)throw Error('Choose an image up to 16 megapixels.');
    const canvas=document.createElement('canvas');canvas.width=256;canvas.height=256;
    const ctx=canvas.getContext('2d');if(!ctx)throw Error('Image processing is unavailable.');
    const edge=Math.min(image.width,image.height);
    ctx.drawImage(image,(image.width-edge)/2,(image.height-edge)/2,edge,edge,0,0,256,256);
    const next={...instanceIcons.value,[id]:canvas.toDataURL('image/webp',.9)};
    try {localStorage.setItem(storageKey,JSON.stringify(next))}catch {throw Error('This device has no space for another icon. Remove an old custom icon and try again.')}
    instanceIcons.value=next;
  }
}
