import {invoke} from '@tauri-apps/api/core';
import type {Api,AppSnapshot,LoginStatus} from './types';
export const desktopApi:Api={
 snapshot:()=>invoke<AppSnapshot>('get_snapshot'),
 importCurrent:(name,replaceId)=>invoke('import_current',{name,replaceId:replaceId??null}),
 rename:(id,name)=>invoke('rename_account',{id,name}),remove:id=>invoke('delete_account',{id}),switchAccount:id=>invoke('switch_account',{id}),
 refresh:id=>id?invoke('refresh_account',{id}):invoke('refresh_all'),
 startLogin:(region,name)=>invoke<LoginStatus>('start_login',{region,name}),loginStatus:taskId=>invoke('get_login_status',{taskId}),cancelLogin:taskId=>invoke('cancel_login',{taskId})
};
