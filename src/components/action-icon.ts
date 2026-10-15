const paths={
 refresh:'<path d="M20 7v5h-5M4 17v-5h5"/><path d="M6.1 7a7 7 0 0 1 11.6-1L20 9M4 15l2.3 3A7 7 0 0 0 17.9 17"/>',
 rename:'<path d="M13 3H5a1 1 0 0 0-1 1v16a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-7M8 7h3M8 11h2M8 17h3"/><path d="m13 11 6.5-6.5a1.4 1.4 0 0 1 2 2L15 13l-3 1 1-3ZM18 6l2 2"/>',
 delete:'<path d="M3 6h18M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2M5 6l1 14a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1l1-14M10 10v7M14 10v7"/>'
};
export type ActionIcon=keyof typeof paths;
export function setActionIcon(button:HTMLButtonElement,icon:ActionIcon,label:string){
 const svg=document.createElementNS('http://www.w3.org/2000/svg','svg');
 for(const [name,value] of Object.entries({viewBox:'0 0 24 24',width:'18',height:'18',fill:'none',stroke:'currentColor','stroke-width':'1.7','stroke-linecap':'round','stroke-linejoin':'round','aria-hidden':'true',focusable:'false'}))svg.setAttribute(name,value);
 svg.innerHTML=paths[icon];
 button.replaceChildren(svg);button.classList.add('icon-button');button.setAttribute('aria-label',label);button.title=label;
}
