// dil: tr
// Clipboard card: only the version is polled, pin and delete from the menu, the time of each copy
const {esit,dogru,q,bek}=RT;
const istek=[];let liste=[],surum=5;const f0=window.fetch;
window.fetch=async(p,a)=>{if(p!=='/api/pano')return f0(p,a);const b=JSON.parse(a.body);istek.push(b);
  if(b.sil){liste=liste.filter(x=>x.m!==b.sil);surum++}
  if(b.sabitle){const o=liste.find(x=>x.m===b.sabitle.m);o.s=b.sabitle.on;liste=[...liste.filter(x=>x.s),...liste.filter(x=>!x.s)];surum++}
  return{json:async()=>({durum:'ok',veri:b.surum===surum?{acik:true,surum}:{acik:true,surum,liste:liste.map(x=>({...x}))}})}};
kartAc('pano',true);await bek(100);
liste=[{m:'birinci',t:Date.now()-120000,s:false},{m:'ikinci',t:Date.now()-3000,s:false}];surum=6;
panoSurum=0;await pano();
esit('rows with times',[...q('#pano').querySelectorAll('.pn')].map(b=>b.textContent),['birinci2 dk önce','ikinciaz önce']);
// nothing changed: the request carries the version, the answer has no list, the card is not redrawn
istek.length=0;const once=q('#pano').innerHTML;await pano();
esit('same version: no list',[istek[0],q('#pano').innerHTML===once],[{surum:6},true]);
// pin from the menu: moves to the top with a pin
const ac=i=>q(`#pano [data-p="${i}"]`).dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:10,clientY:10}));
const sec=t=>[...q('#menu').querySelectorAll('button')].find(b=>b.textContent===t);
ac(1);await bek(30);sec('Sabitle').click();await bek(50);
esit('pinned on top',[...q('#pano').querySelectorAll('.pn')].map(b=>[b.textContent.startsWith('📌 ikinci'),b.classList.contains('sabit')]),[[true,true],[false,false]]);
ac(0);await bek(30);esit('pinned: unpin offered',!!sec('Sabitlemeyi kaldır'),true);q('#menu').hidePopover();
// delete
ac(1);await bek(30);sec('Sil').click();await bek(50);
esit('deleted',[...q('#pano').querySelectorAll('.pn span')].map(s=>s.textContent),['📌 ikinci']);
// "Clear" keeps the pinned ones; closing the card clears all
istek.length=0;await panoAyar(true);esit('clear: unpinned only',istek[0].temizle,true);
istek.length=0;kartAc('pano',false);await bek(100);esit('card closed: all',istek.at(-1)?.temizle,'hepsi');
