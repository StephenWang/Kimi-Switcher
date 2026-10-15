// @vitest-environment jsdom
import {it,expect} from 'vitest';
import {renderAccountCard} from '../components/account-card';
import type {AccountView} from '../types';
const account:AccountView={id:'a',name:'账号A',isCurrent:true,region:'mainland-cn',quota:{week:{usedRatio:1.2,resetAt:null},fiveHour:null,plan:null,fetchedAt:'2026-10-14T00:00:00Z'},status:'error',error:'网络超时'};
it('places the current badge outside the account header',()=>{const card=renderAccountCard(account);expect(card.querySelector('.account-header')?.textContent).not.toContain('当前');expect(card.querySelector('.current-badge')?.textContent).toContain('当前');});
it('shows real current status, missing plan and cached quota after an error',()=>{const card=renderAccountCard(account);expect(card.textContent).toContain('当前');expect(card.textContent).toContain('套餐信息暂不可用');expect(card.textContent).toContain('120%');expect(card.textContent).toContain('网络超时');expect(card.querySelector('[role=progressbar]')?.getAttribute('aria-valuenow')).toBe('100');});
it('disables repeated actions while busy and never renders account name as markup',()=>{const card=renderAccountCard({...account,name:'<img src=x onerror=alert(1)>'},0,true);expect(card.querySelector('img')).toBeNull();expect([...card.querySelectorAll('button')].every(b=>b.disabled)).toBe(true);expect(card.querySelectorAll('button').length).toBeGreaterThan(0);});
it('switches through a user action and does not show stale current state',()=>{let action='';const card=renderAccountCard({...account,isCurrent:false},1,false,(a,id)=>{action=`${a}:${id}`;});const button=[...card.querySelectorAll('button')].find(b=>b.textContent==='切换');button?.click();expect(action).toBe('switch:a');expect(card.textContent).not.toContain('当前');});
