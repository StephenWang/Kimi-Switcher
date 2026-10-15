import type { QuotaWindow } from '../types';
export function formatQuota(ratio: number | null): string { return ratio===null||!Number.isFinite(ratio)?'暂无数据':`${Math.round(Math.max(0,ratio)*100)}%`; }
export function formatReset(value: string | null, locale = 'zh-CN', timeZone?: string): string {
 if(!value||Number.isNaN(Date.parse(value)))return '重置时间未知';
 const parts=new Intl.DateTimeFormat(locale,{timeZone,month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit',hourCycle:'h23'}).formatToParts(new Date(value));
 const get=(type:string)=>parts.find(p=>p.type===type)?.value??'';
 return `重置 ${get('month')}-${get('day')} ${get('hour')}:${get('minute')}`;
}
export function quotaBar(label: string, quota: QuotaWindow | null): HTMLElement {
 const row=document.createElement('div');row.className='quota-row';
 const name=document.createElement('span');name.className='quota-label';name.textContent=label;
 const track=document.createElement('div');track.className='quota-track';track.setAttribute('role','progressbar');track.setAttribute('aria-label',`${label}额度已使用`);track.setAttribute('aria-valuemin','0');track.setAttribute('aria-valuemax','100');
 const fill=document.createElement('div');fill.className=`quota-fill ${label==='7天'?'weekly':'hourly'}${quota&&quota.usedRatio>=0.9?' critical':''}`;
 const value=quota?.usedRatio??null;const percent=value===null?0:Math.max(0,Math.min(100,value*100));
 fill.style.width=`${percent}%`;if(value!==null)track.setAttribute('aria-valuenow',String(percent));else track.setAttribute('aria-valuetext','暂无数据');
 const text=document.createElement('span');text.className=`quota-value${value===null?' unavailable':''}`;text.textContent=formatQuota(value);
 track.append(fill,text);const reset=document.createElement('span');reset.className='quota-reset';reset.textContent=formatReset(quota?.resetAt??null);reset.title=quota?.resetAt??'';
 row.append(name,track,reset);return row;
}
