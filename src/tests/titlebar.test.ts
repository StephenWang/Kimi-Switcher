// @vitest-environment jsdom
import {it,expect,vi} from 'vitest';
import {createTitlebar,type WindowControls} from '../components/titlebar';

it('dispatches window controls and reflects maximize/restore state',async()=>{
 let maximized=false;let resized=()=>{};
 const controls:WindowControls={minimize:vi.fn(async()=>{}),close:vi.fn(async()=>{}),startDragging:vi.fn(async()=>{}),isMaximized:async()=>maximized,toggleMaximize:vi.fn(async()=>{maximized=!maximized;}),onResized:async handler=>{resized=handler;return ()=>{};}};
 const bar=await createTitlebar(controls);
 bar.querySelector<HTMLButtonElement>('[aria-label="最小化"]')!.click();
 bar.querySelector<HTMLButtonElement>('[aria-label="最大化"]')!.click();
 await vi.waitFor(()=>expect(bar.querySelector('[aria-label="还原"]')).not.toBeNull());
 maximized=false;resized();
 await vi.waitFor(()=>expect(bar.querySelector('[aria-label="最大化"]')).not.toBeNull());
 bar.querySelector<HTMLButtonElement>('[aria-label="关闭"]')!.click();
 expect(controls.minimize).toHaveBeenCalledOnce();expect(controls.close).toHaveBeenCalledOnce();
 bar.querySelector('.titlebar-drag')!.dispatchEvent(new MouseEvent('mousedown',{button:0,bubbles:true,detail:1}));
 expect(controls.startDragging).toHaveBeenCalledOnce();
 bar.querySelector('.titlebar-drag')!.dispatchEvent(new MouseEvent('mousedown',{button:0,bubbles:true,detail:2}));
 await vi.waitFor(()=>expect(controls.toggleMaximize).toHaveBeenCalledTimes(2));
 bar.querySelector('button')!.dispatchEvent(new MouseEvent('mousedown',{button:0,bubbles:true,detail:1}));
 expect(controls.startDragging).toHaveBeenCalledOnce();
});

it('disables native controls in browser preview',async()=>{
 const bar=await createTitlebar();
 expect(bar.textContent).toContain('Kimi Switch');
 expect([...bar.querySelectorAll('button')].every(button=>button.disabled)).toBe(true);
});
