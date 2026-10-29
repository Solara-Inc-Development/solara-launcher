import {ref,watch,type Ref} from 'vue';
import {native} from '../../packages/shared/client';
export interface Assignment {id:string;serverId:string;name:string;hostname:string|null;endpoint:string;port:number;status:string;lastError:string|null;updatedAt:string|null}
export function useAssignments(account:Ref<any>){
  const items=ref<Assignment[]>([]),loading=ref(false),error=ref('');
  let requestId=0;
  watch(()=>account.value?.id,()=>{requestId++;items.value=[];loading.value=false;error.value=''});
  async function refresh(){
    const owner=account.value?.id,request=++requestId;
    if(!owner){items.value=[];loading.value=false;error.value='';return}
    loading.value=true;error.value='';
    try{
      const data=await native('solara_request',{method:'GET',path:'/v1/launcher/assignments',body:null});
      if(request!==requestId||account.value?.id!==owner)return;
      if(!Array.isArray(data.assignments))throw Error('The address service returned invalid data.');
      const rows:Assignment[]=data.assignments.slice(0,100).map((row:any)=>{
        if(!/^[a-f0-9]{32}$/.test(row.id)||typeof row.name!=='string'||typeof row.endpoint!=='string'||!Number.isInteger(row.port))throw Error('The address service returned invalid data.');
        return {id:row.id,serverId:String(row.serverId||''),name:row.name,hostname:typeof row.hostname==='string'?row.hostname:null,endpoint:row.endpoint,port:row.port,status:['active','pending','error'].includes(row.status)?row.status:'unknown',lastError:typeof row.lastError==='string'?row.lastError:null,updatedAt:typeof row.updatedAt==='string'?row.updatedAt:null};
      });
      items.value=rows;
    }catch(e){if(request===requestId&&account.value?.id===owner)error.value=String(e instanceof Error?e.message:e)}
    finally{if(request===requestId)loading.value=false}
  }
  return {items,loading,error,refresh};
}
