// dil: en
// sure: 25000
// Accessibility: contrast in all 16 themes, and every button / field has a name a screen reader can read
const {esit,q,bek}=RT;
const lum=h=>{const c=[1,3,5].map(i=>parseInt(h.slice(i,i+2),16)/255).map(x=>x<=.03928?x/12.92:((x+.055)/1.055)**2.4);return .2126*c[0]+.7152*c[1]+.0722*c[2]};
const kar=(a,b)=>{const x=lum(a),y=lum(b);return (Math.max(x,y)+.05)/(Math.min(x,y)+.05)};
const hex=h=>h.length===4?'#'+[1,2,3].map(i=>h[i]+h[i]).join(''):h;
const mix=(a,b)=>'#'+[1,3,5].map(i=>Math.round((parseInt(a.slice(i,i+2),16)+parseInt(b.slice(i,i+2),16))/2).toString(16).padStart(2,'0')).join('');
const dusuk=[];
for(const [v,t] of Object.entries(TEMALAR)){
  const [bg,,,tx,mu,,ye]=t.r.map(hex),cz=t.cz||(v===''?'#e9ecf1':null),ad=v||'light';
  if(kar(tx,bg)<7)dusuk.push(`${ad}: text ${kar(tx,bg).toFixed(2)}`);
  for(const [n,b] of [['background',bg],['glass',cz]])if(b&&!t.k&&kar(mu,b)<4.5||n==='background'&&kar(mu,b)<4.5)dusuk.push(`${ad}: muted on ${n} ${kar(mu,b).toFixed(2)}`);
  if(kar(mix(ye,tx),bg)<3)dusuk.push(`${ad}: focus ring ${kar(mix(ye,tx),bg).toFixed(2)}`);
}
esit('contrast in every theme',dusuk,[]);

// accessible names: visible buttons and fields in the main page, every card's settings and every settings category
const adsiz=new Set();
const tara=etiket=>{for(const e of document.querySelectorAll('button,input:not([type=hidden]),select,textarea,[role=button],[role=switch]')){
  if(!e.offsetParent&&e.type!=='file')continue;if(e.type==='file')continue;
  const lbl=e.id&&document.querySelector(`label[for="${e.id}"]`);
  const n=(e.getAttribute('aria-label')||e.getAttribute('aria-labelledby')&&document.getElementById(e.getAttribute('aria-labelledby'))?.textContent||e.title||e.textContent||e.placeholder||e.closest('label')?.textContent||lbl?.textContent||e.value&&e.type==='submit'||'').toString().trim();
  if(!n)adsiz.add(etiket+': '+e.outerHTML.slice(0,90))}};
document.querySelectorAll('dialog[open]').forEach(d=>d.close());
for(const k of ['kur','sistem','pano','mail'])kartAc(k,true);await bek(300);tara('main');
for(const k of Object.keys(KART_AD)){kartAyar(k);await bek(60);tara('card settings '+k);q('dialog[open] [data-kapat]')?.click();await bek(250)}
ayarAc();await bek(300);for(const b of document.querySelectorAll('.ayNav [data-b]')){b.click();await bek(60);tara('settings '+b.dataset.b)}
document.querySelectorAll('dialog[open]').forEach(d=>d.close());
esit('every control has a name',[...adsiz],[]);
