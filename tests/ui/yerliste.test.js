// dil: en
// sure: 20000
// Country → province → district picker: lists from /api/yerler, the coordinates by the place id, saved like a searched place
const {esit,dogru,q,bek}=RT;
const f0=window.fetch;let cfg=null,idHata=false;
const IL=[{kod:'48',id:'304183',ad:'Muğla'},{kod:'07',id:'323776',ad:'Antalya'}],ILCE=[{kod:'8631961',id:'8631961',ad:'Köyceğiz'},{kod:'7733242',id:'7733242',ad:'Bodrum'}];
const KY={ad:'Köyceğiz',bolge:'Muğla',ulke:'Türkiye',lat:36.978,lon:28.724},BD={ad:'Bodrum',bolge:'Muğla',ulke:'Türkiye',lat:37.04,lon:27.43};
window.fetch=async(p,a)=>{
  const ok=v=>({json:async()=>({durum:'ok',veri:v})});
  if(p==='/api/yerler')return ok(['DE','TR']);
  if(p==='/api/yerler?ulke=TR')return ok(IL);
  if(p==='/api/yerler?ulke=DE')return ok([{kod:'02',id:'2951839',ad:'Bayern'}]);
  if(p==='/api/yerler?ulke=TR&il=48')return ok(ILCE);
  if(p.startsWith('/api/yerler?ulke='))return ok([]);
  if(p==='/api/yerler?id=8631961')return idHata?{json:async()=>({durum:'Could not reach the server'})}:ok(KY);
  if(p==='/api/yerler?id=7733242')return ok(BD);
  if(p==='/api/yerler?id=323776')return ok({ad:'Antalya',bolge:'',ulke:'Türkiye',lat:36.9,lon:30.7});
  if(p==='/api/config'&&a?.method==='POST'){cfg=JSON.parse(a.body);return ok(null)}
  if(p.startsWith('/api/weather?'))return f0('/api/weather');
  if(p==='/api/state'&&a?.method==='POST')return ok(null);
  return f0(p,a)};
await bek(300);
const sec=(n,v)=>{const s=q(`#sor2 [name=${n}]`);s.value=v;s.dispatchEvent(new Event('change'))};
const secenek=n=>[...q(`#sor2 [name=${n}]`).options].map(o=>o.textContent);

// main city: Türkiye is the default, provinces sorted, districts after a province
let p=listedenSec(false);await bek(300);
esit('picker: Türkiye default',[q('#sor2 [name=u]').value,/^(Turkey|Türkiye)$/.test(q('#sor2 [name=u]').selectedOptions[0].textContent)],['TR',true]);
esit('picker: provinces sorted',secenek('i'),['Antalya','Muğla']);
sec('i','304183');await bek(200);
esit('picker: districts of the province',secenek('c'),['—','Bodrum','Köyceğiz']);
sec('c','8631961');q('#sor2 form').requestSubmit();await p;
esit('picker: main city saved',cfg?.konum,KY);

// the province alone is enough (no district chosen)
cfg=null;p=listedenSec(false);await bek(300);sec('i','323776');await bek(200);q('#sor2 form').requestSubmit();await p;
esit('picker: province alone',cfg?.konum?.ad,'Antalya');

// a country without districts: the district row hides
p=listedenSec(false);await bek(300);sec('u','DE');await bek(300);
esit('picker: no districts, row hidden',[secenek('i'),q('#sor2 [name=c]').closest('label').hidden],[['Bayern'],true]);
q('#sor2 [data-kapat]').click();await p;
esit('picker: cancel saves nothing, country remembered',[cfg?.konum?.ad,A('yerUlke')],['Antalya','TR']);

// no network for the coordinates: nothing saved, a message
cfg=null;idHata=true;p=listedenSec(false);await bek(300);sec('i','304183');await bek(200);sec('c','8631961');q('#sor2 form').requestSubmit();await p;
esit('picker: no network',[cfg,q('#tost span')?.textContent],[null,'Could not reach the server']);
idHata=false;

// extra cities: added once, not the main city, up to 4
S.ayar.sehirler=[];await ekSehirEkle(BD);await ekSehirEkle(BD);
esit('extra city added once',S.ayar.sehirler,[{ad:'Bodrum',bolge:'Muğla',lat:37.04,lon:27.43}]);
S.ayar.sehirler=[];await ekSehirEkle(KY); // the demo's main city is Köyceğiz, Muğla
esit('main city not added as extra',S.ayar.sehirler,[]);
S.ayar.sehirler=['A','B','C','D'];await ekSehirEkle(BD);
esit('extra cities: up to 4',S.ayar.sehirler.length,4);

// the button in the weather card settings
S.ayar.sehirler=[];kartAyar('hava');await bek(300);
dogru('card settings: list buttons',!!q('#sor [data-liste="sehir"]')&&!!q('#sor [data-liste="sehirler"]'));
q('#sor [data-liste="sehirler"]').click();await bek(300);sec('i','304183');await bek(200);sec('c','7733242');q('#sor2 form').requestSubmit();await bek(500);
esit('card settings: extra city from the list',[S.ayar.sehirler.map(y=>y.ad),q('#sor [data-a="sehirler"]').value],[['Bodrum'],'Bodrum (Muğla)']);
