export interface WindowControls {
 minimize():Promise<void>;
 toggleMaximize():Promise<void>;
 close():Promise<void>;
 startDragging():Promise<void>;
 isMaximized():Promise<boolean>;
 onResized(handler:()=>void):Promise<()=>void>;
}

const icons={
 minimize:'<path d="M4 8h8"/>',
 maximize:'<rect x="4" y="4" width="8" height="8"/>',
 restore:'<path d="M6 4V2h8v8h-2"/><rect x="2" y="6" width="8" height="8"/>',
 close:'<path d="m4 4 8 8m0-8-8 8"/>'
};

export async function createTitlebar(window?:WindowControls):Promise<HTMLElement>{
 const bar=document.createElement('header');bar.className='window-titlebar';
 const drag=document.createElement('div');drag.className='titlebar-drag';drag.textContent='Kimi Switch';
 const actions=document.createElement('div');actions.className='window-controls';
 const status=document.createElement('span');status.className='window-error';status.setAttribute('role','alert');
 const run=async(action:()=>Promise<void>)=>{try{await action();status.textContent='';}catch{status.textContent='窗口操作失败，请重试';}};
 const button=(label:string,icon:string,action:()=>Promise<void>,className='')=>{
  const element=document.createElement('button');element.type='button';element.className=`window-control ${className}`;
  element.setAttribute('aria-label',label);element.title=window?label:`${label}（桌面应用可用）`;element.disabled=!window;
  element.innerHTML=`<svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.2" aria-hidden="true">${icon}</svg>`;
  element.onclick=()=>void run(action);return element;
 };
 const maximize=button('最大化',icons.maximize,async()=>{await window!.toggleMaximize();await sync();});
 const sync=async()=>{const maximized=await window!.isMaximized();const label=maximized?'还原':'最大化';maximize.setAttribute('aria-label',label);maximize.title=label;maximize.querySelector('svg')!.innerHTML=maximized?icons.restore:icons.maximize;};
 actions.append(button('最小化',icons.minimize,()=>window!.minimize()),maximize,button('关闭',icons.close,()=>window!.close(),'window-close'));
 // Handle the second press before starting a native move loop, as Tauri's drag region does.
 drag.onmousedown=event=>{if(!window||event.button!==0)return;if(event.detail===2)void run(async()=>{await window.toggleMaximize();await sync();});else if(event.detail===1)void run(()=>window.startDragging());};
 bar.append(drag,status,actions);
 if(window){await run(sync);await run(async()=>{await window.onResized(()=>void run(sync));});}
 return bar;
}
