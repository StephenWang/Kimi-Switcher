/// <reference types="vite/client" />
import './styles.css';
import {desktopApi} from './api';
import {AppUI} from './app';
import {isTauri} from '@tauri-apps/api/core';
import {getCurrentWindow} from '@tauri-apps/api/window';
import {createTitlebar} from './components/titlebar';
async function main(){let api=desktopApi;let preview=false;if(import.meta.env.DEV&&new URLSearchParams(location.search).has('preview')){api=(await import('./preview')).createPreviewApi();preview=true;}await new AppUI(document.querySelector('#app')!,api,preview).initialize();}
async function start(){document.body.prepend(await createTitlebar(isTauri()?getCurrentWindow():undefined));await main();}
void start();
