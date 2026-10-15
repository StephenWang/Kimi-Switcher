export interface Field {name:string;label:string;value?:string;options?:{value:string;label:string}[]}
export function askDialog(title:string,description:string,fields:Field[]=[],confirm='确定',danger=false):Promise<Record<string,string>|null>{
 const previous=document.activeElement as HTMLElement|null;
 return new Promise(resolve=>{
  const dialog=document.createElement('dialog');dialog.className='app-dialog';
  const form=document.createElement('form');form.method='dialog';
  const heading=document.createElement('h2');heading.textContent=title;heading.id='dialog-title';dialog.setAttribute('aria-labelledby',heading.id);
  const desc=document.createElement('p');desc.textContent=description;form.append(heading,desc);
  for(const field of fields){const label=document.createElement('label');label.textContent=field.label;let input:HTMLInputElement|HTMLSelectElement;
   if(field.options){input=document.createElement('select');for(const option of field.options){const el=document.createElement('option');el.value=option.value;el.textContent=option.label;input.append(el);}}else{input=document.createElement('input');input.maxLength=40;input.required=true;input.autocomplete='off';}
   input.name=field.name;input.value=field.value??field.options?.[0]?.value??'';label.append(input);form.append(label);
  }
  const actions=document.createElement('div');actions.className='dialog-actions';const cancel=document.createElement('button');cancel.type='button';cancel.textContent='取消';const submit=document.createElement('button');submit.type='submit';submit.className=danger?'danger':'primary';submit.textContent=confirm;actions.append(cancel,submit);form.append(actions);dialog.append(form);document.body.append(dialog);
  let settled=false;const finish=(data:Record<string,string>|null)=>{if(settled)return;settled=true;dialog.close();dialog.remove();previous?.focus();resolve(data);};
  cancel.onclick=()=>finish(null);dialog.addEventListener('cancel',e=>{e.preventDefault();finish(null);});form.onsubmit=e=>{e.preventDefault();if(!form.reportValidity())return;const data=Object.fromEntries(new FormData(form).entries()) as Record<string,string>;if(Object.values(data).some(v=>!v.trim()))return;finish(data);};
  dialog.showModal();(form.querySelector('input,select') as HTMLElement|null)?.focus();
 });
}
