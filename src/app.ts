import type {Api,AppSnapshot,LoginStatus} from './types';
import {renderAccountCard} from './components/account-card';
import {askDialog} from './components/dialogs';
import sunIcon from './assets/sun.svg?url';
import moonIcon from './assets/moon.svg?url';
export class AppUI {
 private data:AppSnapshot={accounts:[],cliVersion:null,cliHome:'',activeState:'loading',error:null};
 private working=false;private notice='';private noticeType='';private login:LoginStatus|null=null;private timer:ReturnType<typeof setTimeout>|undefined;
 constructor(private root:HTMLElement,private api:Api,private preview=false){document.documentElement.dataset.theme=localStorage.getItem('kimi-theme')??'dark';}
 async initialize(){this.render();try{this.data=await this.api.snapshot();}catch{this.data.error='请在桌面应用中使用账号管理功能。开发预览需显式指定 preview 参数。';this.data.activeState='unavailable';}this.render();}
 private button(text:string,action:()=>void,className='',disabled=false){const b=document.createElement('button');b.type='button';b.textContent=text;b.className=className;b.disabled=disabled;b.onclick=action;return b;}
 private get loginActive(){return !!this.login&&['starting','waiting','saving','cancelling'].includes(this.login.stage);}
 private render(){
  const busy=this.working||this.loginActive;this.root.replaceChildren();
  const toolbar=document.createElement('header');toolbar.className='toolbar';const title=document.createElement('h1');title.textContent='Kimi Switch';const actions=document.createElement('div');actions.className='toolbar-actions';
  const unavailable=!this.data.cliVersion;
  const light=document.documentElement.dataset.theme==='light';
  const themeLabel=light?'切换到深色模式':'切换到浅色模式';
  const themeButton=this.button('',()=>{const focused=document.activeElement===themeButton;const theme=light?'dark':'light';document.documentElement.dataset.theme=theme;localStorage.setItem('kimi-theme',theme);this.render();if(focused)this.root.querySelector<HTMLButtonElement>('.theme-toggle')?.focus();},'theme-toggle');
  themeButton.setAttribute('aria-label',themeLabel);themeButton.title=themeLabel;
  const themeIcon=document.createElement('img');themeIcon.src=light?moonIcon:sunIcon;themeIcon.alt='';themeIcon.setAttribute('aria-hidden','true');themeIcon.width=20;themeIcon.height=20;themeButton.append(themeIcon);
  actions.append(this.button('＋ 添加账号',()=>void this.add(),'primary',busy||unavailable),this.button('导入当前账号',()=>void this.importCurrent(),'',busy||unavailable),this.button(this.working?'处理中…':'刷新',()=>void this.run(()=>this.api.refresh(),'额度已刷新'),'',busy||unavailable),themeButton);
  toolbar.append(title,actions);this.root.append(toolbar);
  if(this.data.error)this.message(this.data.error,'error');
  if(this.notice)this.message(this.notice,this.noticeType);
  if(this.data.activeState==='unimported')this.message('当前 CLI 账号尚未导入，可点击“导入当前账号”保存。','');
  if(this.login)this.renderLogin(this.login);
  const list=document.createElement('main');list.className='accounts';list.setAttribute('aria-label','账号列表');
  if(this.data.accounts.length){this.data.accounts.forEach((account,index)=>list.append(renderAccountCard(account,index,busy,(action,id)=>void this.accountAction(action,id))));}
  else{const empty=document.createElement('section');empty.className='empty-state';const h=document.createElement('h2');h.textContent=this.data.activeState==='loading'?'正在读取账号…':'还没有保存账号';const p=document.createElement('p');p.textContent='添加一个 Kimi Code CLI 账号，或导入已登录的账号。\n账号会加密保存在这台电脑上。';p.style.whiteSpace='pre-line';const buttons=document.createElement('div');buttons.className='empty-actions';buttons.append(this.button('添加账号',()=>void this.add(),'primary',busy||unavailable),this.button('导入当前账号',()=>void this.importCurrent(),'',busy||unavailable));empty.append(h,p,buttons);list.append(empty);}
  this.root.append(list);
  const footer=document.createElement('footer');footer.className='app-footer';const cli=document.createElement('span');cli.textContent=this.preview?'界面预览 · 合成数据':`Kimi Code CLI ${this.data.cliVersion??'未检测到'}`;if(this.preview)cli.className='preview-badge';cli.title=this.data.cliHome;const hint=document.createElement('span');hint.textContent='切换前请关闭 CLI 会话 · 切换后重新启动 kimi';footer.append(cli,hint);this.root.append(footer);
 }
 private message(text:string,type:string){const el=document.createElement('div');el.className=`notice ${type}`;el.textContent=text;el.setAttribute('role',type==='error'?'alert':'status');this.root.append(el);}
 private async run(operation:()=>Promise<AppSnapshot>,success:string){if(this.working||this.loginActive)return;this.working=true;this.notice='';this.render();try{this.data=await operation();this.notice=success;this.noticeType='success';}catch(e){this.notice=errorMessage(e);this.noticeType='error';try{this.data=await this.api.snapshot();}catch{/* Keep last visible data. */}}finally{this.working=false;this.render();}}
 private async importCurrent(){
  const result=await askDialog('导入当前账号','读取 Kimi Code CLI 当前登录信息，并加密保存为一个账号。',[{name:'name',label:'账号名称',value:`账号${this.data.accounts.length+1}`}],'导入');if(!result)return;
  let replaceId:string|undefined;
  // Unknown identity is resolved explicitly, never silently merged.
  await this.run(async()=>{try{return await this.api.importCurrent(result.name,replaceId);}catch(e){if((e as {code?:string})?.code!=='identity_choice')throw e;const choice=await askDialog('确认账号身份','CLI 凭据未提供稳定账号身份，请选择新建，或更新已有账号。',[{name:'replace',label:'保存方式',options:[{value:'new',label:'新建账号'},...this.data.accounts.map(a=>({value:a.id,label:`更新 ${a.name}`}))]}],'继续');if(!choice)throw {message:'已取消导入'};replaceId=choice.replace;return this.api.importCurrent(result.name,replaceId);}},'当前账号已加密保存');
 }
 private async accountAction(action:string,id:string){
  const account=this.data.accounts.find(a=>a.id===id);if(!account)return;
  if(action==='refresh'){await this.run(()=>this.api.refresh(id),`${account.name}额度已刷新`);return;}
  if(action==='switch'){await this.run(()=>this.api.switchAccount(id),`已切换到${account.name}，请重新启动 kimi`);return;}
  if(action==='rename'){const result=await askDialog('重命名账号','名称仅用于在助手中区分账号。',[{name:'name',label:'账号名称',value:account.name}],'保存');if(result)await this.run(()=>this.api.rename(id,result.name),'账号名称已更新');return;}
  if(action==='delete'){const result=await askDialog('删除账号',`确定删除“${account.name}”吗？这将移除助手保存的登录信息，CLI 当前登录仍会保留。`,[],'删除',true);if(result)await this.run(()=>this.api.remove(id),'已删除助手保存的账号');}
 }
 private async add(){
  const result=await askDialog('添加 Kimi Code 账号','将通过官方 CLI 打开浏览器授权。登录完成后返回这里，当前账号保持不变。',[{name:'name',label:'账号名称',value:`账号${this.data.accounts.length+1}`},{name:'region',label:'登录地区',options:[{value:'mainland-cn',label:'中国大陆 · kimi.com'},{value:'global',label:'全球 · kimi.ai'}]}],'开始登录');if(!result)return;
  this.working=true;this.notice='';this.render();try{this.login=await this.api.startLogin(result.region,result.name);this.pollLogin();}catch(e){this.notice=errorMessage(e);this.noticeType='error';}finally{this.working=false;this.render();}
 }
 private pollLogin(){if(this.timer)clearTimeout(this.timer);this.timer=setTimeout(async()=>{if(!this.login)return;try{this.login=await this.api.loginStatus(this.login.taskId);if(this.login.stage==='complete'){this.data=await this.api.snapshot();this.notice='新账号已保存，当前 CLI 账号保持不变';this.noticeType='success';this.login=null;}else if(this.loginActive)this.pollLogin();}catch(e){this.notice=errorMessage(e);this.noticeType='error';this.login=null;}this.render();},700);}
 private renderLogin(login:LoginStatus){const panel=document.createElement('section');panel.className='login-panel';panel.setAttribute('aria-live','polite');const h=document.createElement('h2');h.textContent=({starting:'正在启动官方登录…',waiting:'请在浏览器中完成授权',saving:'正在加密保存账号…',cancelling:'正在取消登录…',cancelled:'登录已取消',failed:'登录未完成'} as Record<string,string>)[login.stage]??'账号登录';const p=document.createElement('p');p.textContent=login.error??'官方 CLI 会自动打开授权页面。如果浏览器未打开，请复制下方链接。';panel.append(h,p);if(login.verificationUrl){const input=document.createElement('input');input.className='login-url';input.readOnly=true;input.value=login.verificationUrl;input.setAttribute('aria-label','官方授权链接');input.onclick=()=>input.select();panel.append(input);}panel.append(this.button(this.loginActive?'取消登录':'关闭',()=>{if(this.loginActive){void this.api.cancelLogin(login.taskId).then(()=>{if(this.login)this.login.stage='cancelling';this.render();}).catch(e=>{this.notice=errorMessage(e);this.noticeType='error';this.render();});}else{this.login=null;this.render();}},'',login.stage==='cancelling'||login.stage==='saving'));this.root.append(panel);}
}
function errorMessage(error:unknown):string{if(typeof error==='object'&&error&&'message'in error&&typeof error.message==='string')return error.message;return '操作失败，请稍后重试';}
