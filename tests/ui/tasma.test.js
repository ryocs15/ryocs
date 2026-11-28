// dil: pl,es
// Translated texts fit: a label from the dictionary is not cut (nowrap/ellipsis/hidden) and does not stick out of its card or window.
// On the dashboard only texts that are dictionary values are checked, so user data cut on purpose (event, app, song names) is left out.
// Checked as is and with settings windows forced narrow (a small or scaled screen), like ayarsatir.
const {dogru,bek}=RT;
const soz=new Set(Object.values(SOZ).map(v=>v.trim()).filter(v=>v.length>2));
const bulunan=new Set();
const bak=(yer,kok)=>{
  for(const e of kok.querySelectorAll('button,label,span,small,b,h1,h2,h3,h4,th,td,a,p,div')){
    if(!e.offsetParent||e.children.length>2)continue;
    const t=(e.innerText||'').trim();if(!t||!soz.has(t)&&!e.closest('dialog'))continue; // settings windows hold no user data: all their text counts (news categories are not in the dictionary)
    const cs=getComputedStyle(e),kes=cs.overflow!=='visible'||cs.textOverflow==='ellipsis'||cs.whiteSpace==='nowrap';
    if(kes&&e.scrollWidth>e.clientWidth+2)bulunan.add(`${yer}: "${t}" cut (${e.scrollWidth}>${e.clientWidth})`);
    const pr=e.parentElement.getBoundingClientRect(),er=e.getBoundingClientRect();
    if(er.right>pr.right+2)bulunan.add(`${yer}: "${t}" sticks out of its box`); // "Umiarkowana" ran out of its weather tile
    const c=e.closest('dialog[open],.kart,[data-kart]');if(!c)continue;
    const r=e.getBoundingClientRect(),cr=c.getBoundingClientRect();
    if(r.right>cr.right+2||r.left<cr.left-2)bulunan.add(`${yer}: "${t}" sticks out of its ${c.tagName==='DIALOG'?'window':'card'}`);
  }
  // a placeholder is cut silently too ("Szukaj w ustawieniac"): measured by putting it in as the value for a moment
  for(const e of kok.querySelectorAll('input[placeholder]')){
    if(!e.offsetParent||e.value||!soz.has(e.placeholder.trim()))continue;
    e.value=e.placeholder;const w=e.scrollWidth>e.clientWidth+2;e.value='';
    if(w)bulunan.add(`${yer}: placeholder "${e.placeholder}" cut`);
  }
};
await bek(800);
// air quality words in a weather tile as narrow as the card makes them (3 tiles per row on a small card); the demo weather has no air quality
{const g=document.createElement('div');g.className='hvDet';g.style.width='260px';document.querySelector('[data-kart=hava]').append(g);
 for(const v of [10,30,50,70,90,120]){g.innerHTML=havaKalite({european_aqi:v,pm2_5:5});bak('air quality '+v,g)}g.remove()}
bak('dashboard',document.querySelector('main'));
for(const gen of ['',420]){
  for(const k of Object.keys(KART_AYAR)){kartAyar(k);const d=document.querySelector('dialog[open]');if(gen)d.style.width=gen+'px';await bek(30);bak(k+(gen&&' '+gen),d);d.close()}
  ayarAc();const d=$('#ayarlar');if(gen)d.style.width=gen+200+'px';
  await bek(50);
  for(const b of document.querySelectorAll('#ayarlar .ayNav [role=tab]')){b.click();await bek(30);bak('settings/'+b.dataset.b+(gen&&' '+gen),d)}
  d.close();d.style.width='';
}
// Spanish month titles: only the first letter is raised (text-transform:capitalize gave "Noviembre De 2026")
RT.esit('calendar title not capitalized word by word',getComputedStyle($('.head h1')).textTransform,'none');
dogru('translated texts fit'+(bulunan.size?': '+[...bulunan].join(' | '):''),!bulunan.size);
