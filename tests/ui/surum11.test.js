// dil: tr
// 1.1: cache in the encrypted file, recovery screen, retry buttons, keyboard resize, moved-event reminder
const {esit,dogru}=RT;
// The cache moves from localStorage (plain text) into onbellek.json once, and is saved there
{const f=window.fetch,gonder=[];window.fetch=(p,o)=>{if(p==='/api/onbellek'&&o?.body)gonder.push(JSON.parse(o.body));return p==='/api/onbellek'&&!o?.body?Promise.resolve({json:async()=>({durum:'ok',veri:{}})}):f(p,o)};
 localStorage.ileri=JSON.stringify([{id:'x',summary:'Eski önbellek',start:{date:'2030-01-01'}}]);localStorage.tCache='[["k",{"items":[]}]]';localStorage.bekleyen='["Hatırlatıcı: çay"]';
 bekleyen=[];await onbYukle();
 esit('moved from localStorage',[ileri[0]?.summary,tCache.has('k'),bekleyen,localStorage.ileri,localStorage.tCache,localStorage.bekleyen],['Eski önbellek',true,['Hatırlatıcı: çay'],undefined,undefined,undefined]);
 await RT.bek(700);window.fetch=f;
 esit('written to the encrypted file',gonder.at(-1)&&[gonder.at(-1).ileri[0].summary,Array.isArray(gonder.at(-1).tCache),Array.isArray(gonder.at(-1).bekleyen)],['Eski önbellek',true,true]);}
// A moved event is reminded again before its new time
{const once=[],f=window.fetch;window.fetch=(p,o)=>{if(p==='/api/notify'&&o.body.includes('Yaklaşan etkinlik'))once.push(o.body);return f(p,o)};S.ayar.etk=true;S.ayar.rahatsiz=false;S.ayar.havaAksam=false;S.ayar.havaUyari=false;
 const t=d=>new Date(Date.now()+d).toISOString();ileri=[{id:'tasinan',summary:'Görüşme',start:{dateTime:t(5*6e4)},end:{dateTime:t(36e5)}}];hatirlat();hatirlat();
 ileri=[{id:'tasinan',summary:'Görüşme',start:{dateTime:t(8*6e4)},end:{dateTime:t(36e5)}}];hatirlat();window.fetch=f;
 esit('once per start time',once.length,2);}
// No data: a recovery screen with the backups, import and folder
await veriKurtar('Veri dosyası bu Windows hesabında açılamıyor');
esit('recovery screen',[...document.querySelectorAll('#kurtar [data-k]')].map(b=>b.dataset.k).filter(k=>k!=='yedek'),['klasor','ice']);
kapat($('#sor'));await RT.bek(800);
// Retry button on a failed card
kurV=null;kurHata=true;kartAc('kur',true);kurCiz();
dogru('retry on rates error',$('#kur button[onclick="hepsi()"]'));
// Alt+Shift+↑ on a focused card makes it taller (the weight drag-resizing uses)
{const el=[...document.querySelectorAll('main [data-kart] button')].find(b=>b.offsetParent&&!b.disabled);const k=kartAd(el.closest('[data-kart]'));
 el.focus();dogru('focused',document.activeElement===el);const once=kb(k);dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowUp',altKey:true,shiftKey:true}));
 esit('keyboard resize',+(kb(k)-once).toFixed(2),.25);}
// OneDrive: the password button only with sync on; the state text says whether the copy is encrypted
esitleYaz({durum:'ok',veri:{durum:'ok',acik:true,klasor:'D:\\OD',parola:false}});
esit('unencrypted copy is mentioned',[$('#aEsPar').hidden,/şifresiz/.test($('#aEsD').textContent)],[false,true]);
esitleYaz({durum:'ok',veri:{durum:'parola',acik:true,klasor:'D:\\OD',parola:false}});
dogru('asks for the password',/parolayı gir/.test($('#aEsD').textContent));
// Weather, the last track and the name leave plain localStorage; only sunrise/sunset stays there for the splash screen
{const f=window.fetch;let gonder=null;window.fetch=(p,o)=>{if(p==='/api/onbellek'&&o?.body)gonder=JSON.parse(o.body);return p==='/api/onbellek'&&!o?.body?Promise.resolve({json:async()=>({durum:'ok',veri:{}})}):f(p,o)};
 localStorage.hava=JSON.stringify({t:Date.now(),v:{sehir:'Köyceğiz',latitude:36.9,longitude:28.6,daily:{time:['2026-11-20'],sunrise:['2026-11-20T07:00'],sunset:['2026-11-20T17:00']}}});localStorage.ad='Deneme';localStorage.sp='{"durum":"ok","veri":{"item":{"id":"t1"}}}';
 havaO=spO=null;adO='';await onbYukle();await RT.bek(700);window.fetch=f;
 esit('moved and removed',[havaO?.v?.sehir,spO?.veri?.item?.id,adO,localStorage.hava,localStorage.ad,localStorage.sp],['Köyceğiz','t1','Deneme',undefined,undefined,undefined]);
 esit('written to the encrypted cache',[gonder?.hava?.v?.latitude,gonder?.sp?.veri?.item?.id,gonder?.ad],[36.9,'t1','Deneme']);
 await hava(true);
 esit('plain storage: only sunrise/sunset',[localStorage.hava,Object.keys(JSON.parse(localStorage.gunes||'{}'))],[undefined,['time','sunrise','sunset']]);}
