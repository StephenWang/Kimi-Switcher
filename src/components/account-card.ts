import type { AccountView } from '../types';
import {quotaBar} from './quota-bar';
export function renderAccountCard(account: AccountView, index = 0, busy = false, onAction: (action: string, id: string) => void = () => {}): HTMLElement {
 const card=document.createElement('article');card.className='account-card';card.dataset.accountId=account.id;
 const header=document.createElement('div');header.className='account-header';
 const identity=document.createElement('div');identity.className='account-identity';
 const marker=document.createElement('span');marker.className=account.isCurrent?'current-marker':'account-number';marker.textContent=account.isCurrent?'● 当前':String(index+1);
 const name=document.createElement('h2');name.textContent=account.name;name.title=account.name;
 const plan=document.createElement('span');plan.className='plan-tag';plan.textContent=account.quota.plan??'套餐信息暂不可用';
 identity.append(marker,name,plan);const actions=document.createElement('div');actions.className='account-actions';
 const buttons:[string,string,string][]=[...(!account.isCurrent?[['switch','切换','primary'] as [string,string,string]]:[]),['refresh','刷新',''],['rename','重命名',''],['delete','删除','danger']];
 for(const [action,label,className] of buttons){const button=document.createElement('button');button.type='button';button.textContent=label;button.className=className;button.disabled=busy;button.setAttribute('aria-label',`${label}${account.name}`);button.onclick=()=>onAction(action,account.id);actions.append(button);}
 header.append(identity,actions);const quotas=document.createElement('div');quotas.className='quota-list';quotas.append(quotaBar('7天',account.quota.week),quotaBar('5小时',account.quota.fiveHour));card.append(header,quotas);
 if(account.error){const error=document.createElement('p');error.className='account-error';error.textContent=`${account.error}${account.quota.fetchedAt?' · 保留上次刷新数据':''}`;error.setAttribute('role','status');card.append(error);}
 if(account.quota.fetchedAt)card.title=`上次刷新：${new Date(account.quota.fetchedAt).toLocaleString()}`;
 return card;
}
