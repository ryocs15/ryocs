// Icon-only buttons without an accessible name
const ad=b=>(b.getAttribute('aria-label')||b.getAttribute('title')||b.textContent||'').trim()||b.getAttribute('aria-labelledby');
for(const k of Object.keys(KART_AYAR||{}))try{kartAc(k,true)}catch{}
await RT.bek(300);
const l=[...document.querySelectorAll('button,[role=button],[role=switch]')].filter(b=>!ad(b)).map(b=>b.outerHTML.slice(0,140));
RT.esit('unnamed',l,[]);
