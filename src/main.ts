/// <reference types="vite/client" />
import './styles.css';
import {desktopApi} from './api';
import {AppUI} from './app';
async function main(){let api=desktopApi;let preview=false;if(import.meta.env.DEV&&new URLSearchParams(location.search).has('preview')){api=(await import('./preview')).createPreviewApi();preview=true;}await new AppUI(document.querySelector('#app')!,api,preview).initialize();}
void main();
