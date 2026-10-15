// Explicit development-only preview. This module is removed from release builds.
import type {Api,AppSnapshot,AccountView,LoginStatus} from './types';
export function createPreviewApi():Api{
 const names=['账号A','账号B','账号C'];const ratios=[[.3,.5],[.33,.71],[.96,0]];
 const accounts:AccountView[]=names.map((name,i)=>({id:String(i),name,isCurrent:i===0,region:'mainland-cn',status:'ready',error:null,quota:{week:{usedRatio:ratios[i][0],resetAt:['2026-09-10T01:01:00+08:00','2026-09-08T11:52:00+08:00','2026-09-06T22:32:00+08:00'][i]},fiveHour:{usedRatio:ratios[i][1],resetAt:['2026-09-04T15:01:00+08:00','2026-09-04T21:52:00+08:00','2026-09-04T16:32:00+08:00'][i]},plan:i===2?'Allegro 全能尊享（Advanced）':'Allegretto 专业优选（Intermediate）',fetchedAt:'2026-10-15T00:00:00Z'}}));
 const mode=new URLSearchParams(location.search).get('preview');
 const data:AppSnapshot={accounts:mode==='empty'?[]:accounts,cliVersion:'2.1.1',cliHome:'开发预览',activeState:'known',error:mode==='error'?'网络连接失败，已保留上次额度数据':null};
 let login:LoginStatus={taskId:'preview-login',stage:'waiting',verificationUrl:'https://auth.kimi.com/device?user_code=PREVIEW',error:null};
 const copy=()=>Promise.resolve(structuredClone(data));
 return {snapshot:copy,importCurrent:async(name)=>{if(!data.accounts.length){const account=structuredClone(accounts[0]);account.name=name;data.accounts.push(account);}return copy();},rename:async(id,name)=>{const a=data.accounts.find(a=>a.id===id);if(a)a.name=name;return copy();},remove:async id=>{data.accounts=data.accounts.filter(a=>a.id!==id);return copy();},switchAccount:async id=>{data.accounts.forEach(a=>a.isCurrent=a.id===id);return copy();},refresh:copy,startLogin:async()=>({...login}),loginStatus:async()=>({...login}),cancelLogin:async()=>{login={...login,stage:'cancelled'};return {...login};}};
}
