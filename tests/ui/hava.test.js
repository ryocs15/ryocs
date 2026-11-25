// dil: en
// sure: 25000
// Weather cities: place picker for same-name places, saving with coordinates, switching without rebuilding the card
const {esit,dogru,q,bek}=RT;
const f0=window.fetch,istek=[];let cfg=null;
const Y=[{ad:'Yeşilova',bolge:'Burdur',ulke:'Türkiye',lat:37.5,lon:29.75},{ad:'Yeşilova',bolge:'Tufanbeyli, Adana',ulke:'Türkiye',lat:38.2,lon:36.2}];
window.fetch=async(p,a)=>{
  if(p.startsWith('/api/sehir_ara?q=')){istek.push(p);const ad=decodeURIComponent(p.split('=')[1]);
    const k=ad.toLocaleLowerCase('tr');
    return{json:async()=>({durum:'ok',veri:k==='yeşilova'?Y:k==='ankara'?[{ad:'Ankara',bolge:'',ulke:'Türkiye',lat:39.9,lon:32.8}]:k==='köyceğiz'?[{ad:'Köyceğiz',bolge:'Muğla',ulke:'Türkiye',lat:36.97,lon:28.68}]:[]})}}
  if(p.startsWith('/api/weather?')&&/[?&](lat|sehir)=/.test(p)){istek.push(p);await bek(120);const u=new URLSearchParams(p.split('?')[1]);
    const r=await (await f0('/api/weather')).json();r.veri=JSON.parse(JSON.stringify(r.veri));
    r.veri.sehir=u.get('ad')||u.get('sehir');r.veri.bolge=u.get('bolge')||'';r.veri.current.temperature_2m=+u.get('lat')||3;return{json:async()=>r}}
  if(p==='/api/config'&&a?.method==='POST'){cfg=JSON.parse(a.body);return{json:async()=>({durum:'ok',veri:cfg.konum?.ad})}}
  if(p==='/api/state'&&a?.method==='POST')return{json:async()=>({durum:'ok'})};
  return f0(p,a)};
await bek(300);

// two places share a name: the picker asks (in its own dialog)
const is=sehirlerKaydet('Yeşilova, Ankara, Yokyer');await bek(400);
esit('picker: question',[q('#sor2 h1')?.textContent,[...(q('#sor2 select')?.options||[])].map(x=>x.textContent)],
  ['Yeşilova: which one?',['Yeşilova, Burdur, Türkiye','Yeşilova, Tufanbeyli, Adana, Türkiye']]);
q('#sor2 select').value='1';q('#sor2 form').requestSubmit();await is;await bek(400);
esit('picker: saved with coordinates',S.ayar.sehirler,[{ad:'Yeşilova',bolge:'Tufanbeyli, Adana',lat:38.2,lon:36.2},{ad:'Ankara',bolge:'',lat:39.9,lon:32.8}]);
esit('picker: unknown city reported',q('#tost span').textContent,'Could not add: Yokyer');
esit('card: city buttons',[...document.querySelectorAll('#hava .hvSehir button')].map(b=>b.textContent),['Köyceğiz','Yeşilova','Ankara']);
const alan=sehirler().map(yerEtiket).join(', ');
esit('settings field shows regions',alan,'Yeşilova (Tufanbeyli / Adana), Ankara');

// saving the same text again asks nothing; "Name (region)" narrows without a question
istek.length=0;const once=JSON.stringify(S.ayar.sehirler);await sehirlerKaydet(alan);
esit('unchanged entries: no new search',[istek.filter(x=>x.includes('sehir_ara')).length,JSON.stringify(S.ayar.sehirler)===once],[0,true]);
await sehirlerKaydet('Yeşilova (Burdur)');
esit('region hint narrows',[S.ayar.sehirler,q('#sor2').open],[[{ad:'Yeşilova',bolge:'Burdur',lat:37.5,lon:29.75}],false]);
await sehirlerKaydet('köyceğiz, YEŞİLOVA (burdur), Yeşilova (Burdur)');
esit('main city and duplicates skipped',S.ayar.sehirler.map(y=>y.bolge),['Burdur']);

// older saves keep a plain name
S.ayar.sehirler=['Ankara'];await havaDiger(true);
esit('older save: name string still works',[...document.querySelectorAll('#hava .hvSehir button')].map(b=>b.textContent),['Köyceğiz','Ankara']);

// main city through the picker
const m=sehirKaydet('Yeşilova');await bek(400);q('#sor2 select').value='0';q('#sor2 form').requestSubmit();await m;
esit('main city saved with its place',cfg,{konum:Y[0]});

// switching cities: the card is not rebuilt, one slide-in, no number pop
S.ayar.anim=true;ayarUygula();
S.ayar.sehirler=[{ad:'Yeşilova',bolge:'Burdur',lat:37.5,lon:29.75}];havaC={};havaSehir='';await havaDiger(true);await bek(900);
const el=q('#hava'),h2=el.querySelector('h2'),seg=el.querySelector('.seg'),ek=q('#havaEk'),govde=el.querySelector('.havaGovde');
let pop=0;new MutationObserver(l=>l.forEach(x=>{if(x.target.classList?.contains('pop'))pop++})).observe(el,{subtree:true,attributes:true,attributeFilter:['class']});
el.querySelector('.hvSehir [data-hs="37.5,29.75"]').click();await bek(30);
const kaydi=a=>a.effect?.getKeyframes?.().some(k=>String(k.transform).includes('translateX'));
dogru('switch: body slides in',govde.getAnimations().some(kaydi)&&ek.getAnimations().some(kaydi));
dogru('switch: no strip fade on top',!el.querySelector('.saatlik.degisti'));
await bek(400);
esit('switch: same card parts',[h2===el.querySelector('h2'),seg===el.querySelector('.seg'),ek===q('#havaEk'),govde===el.querySelector('.havaGovde')],[true,true,true,true]);
esit('switch: title and pressed button',[el.querySelector('.hvAd').textContent,[...el.querySelectorAll('.hvSehir button')].map(b=>b.getAttribute('aria-pressed'))],['Yeşilova',['false','true']]);
esit('switch: no number pop',pop,0);
// the top line and the clock card follow the shown city at once, with an opacity-only fade (nothing slides there)
dogru('switch: top line follows',q('#gunOzet').textContent.includes('Yeşilova 38°'));
dogru('switch: sunrise follows',!havaC['37.5,29.75'].v.daily?.sunrise?.[0]||q('#gunes span').title.endsWith(' · Yeşilova'));
const n0=q('#gunOzet').getAnimations().length; // headless: the timeline may not advance, so earlier fades can still be listed
el.querySelector('.hvSehir [data-hs=""]').click();await bek(30);
const solma=q('#gunOzet').getAnimations();
esit('switch: top line fades only',[solma.length-n0,solma.some(kaydi)],[1,false]);
await bek(400);
esit('switch back',[el.querySelector('.hvAd').textContent,h2===el.querySelector('h2'),q('#gunOzet').textContent.includes('Köyceğiz '),q('#saatEk h3')?.textContent.includes('Köyceğiz')??true],['Köyceğiz',true,true,true]);
const n1=q('#gunOzet').getAnimations().length;tick();await bek(1100);dogru('next ticks: no new fade',q('#gunOzet').getAnimations().length<=n1); // older fades may finish meanwhile
// forecast times are the place's own clock
esit('place time → moment',[yerAn({utc_offset_seconds:3600},'2026-11-21T08:00').toISOString(),+yerAn({},'2026-11-21T08:00')],['2026-11-21T07:00:00.000Z',+new Date('2026-11-21T08:00')]);

// a city not loaded yet: only it is fetched, its button shows loading, extra clicks are ignored
S.ayar.sehirler.push({ad:'Yeşilova',bolge:'Tufanbeyli, Adana',lat:38.2,lon:36.2});delete havaC['38.2,36.2'];havaCiz();
istek.length=0;const c2=el.querySelector('[data-hs="38.2,36.2"]');c2.click();await bek(40);
dogru('loading state',c2.classList.contains('yuk'));c2.click();await bek(400);
esit('after loading',[havaSehir,c2.classList.contains('yuk'),istek.length],['38.2,36.2',false,1]);

// the picker opens above the card settings, which stay open
kartAyar('hava');await bek(300);const inp=q('#sor [data-a="sehirler"]');
inp.value='Yeşilova (Tufanbeyli / Adana), Yeşilova';inp.dispatchEvent(new Event('change',{bubbles:true}));await bek(400);
esit('picker above settings',[q('#sor').open,q('#sor2').open],[true,true]);
q('#sor2 select').value='0';q('#sor2 form').requestSubmit();await bek(900);
esit('settings stay, field updated',[q('#sor').open,q('#sor2').open,inp.isConnected,inp.value],[true,false,true,'Yeşilova (Tufanbeyli / Adana), Yeşilova (Burdur)']);

// 1.1.6: night icons, another time zone's hours, "rain today" wording
esit('night: moon for clear and partly cloudy, older data: sun',[havaTur(0,true),havaTur(2,true),havaTur(3,true),havaTur(0,false),havaTur(0,undefined)],['ay','ayBulut','bulut','gunes','gunes']);
{const w=JSON.parse(JSON.stringify(havaV)),n=new Date(),saat=x=>new Date(+n+x*1e3).toISOString().slice(0,13);
  w.utc_offset_seconds=-5*3600;w.sehir='New York';w.current.time=saat(-5*3600-7200)+':00';w.current.precipitation=0;
  w.hourly={time:Array.from({length:30},(_,k)=>new Date(+n+(k-3)*36e5-5*36e5).toISOString().slice(0,13)+':00'),temperature_2m:Array(30).fill(5),precipitation_probability:Array(30).fill(0),weather_code:Array(30).fill(0),is_day:Array(30).fill(0)};
  w.daily.precipitation_probability_max[0]=70;
  havaC['ny']={t:Date.now(),v:w};S.ayar.sehirler=[{ad:'New York',lat:40.7,lon:-74}];havaSehir='ny';S.ayar.havaGun=false;havaCiz();await bek(50);
  const ilk=q('#hava .saatlik div:nth-child(2) small').textContent;
  esit('other time zone: strip starts at that city\'s hour',ilk,SAAT12()?String((+saat(-5*3600+7200).slice(11,13))%12||12)+ampm(+saat(-5*3600+7200).slice(11,13)):saat(-5*3600+7200).slice(11,13));
  dogru('night hours: moon icons',!!q('#hava .saatlik svg.hi-ay'));
  dogru('rain wording: today',q('#hava .now .m').textContent.includes('rain today 70'));
  havaSehir='';havaCiz();}

// 1.1.14: a city saved by name only (Köyceğiz: Şırnak's came first) is asked once; a saved place is not
{const f1=window.fetch;let g={sehir:'Yeşilova',konum:false};cfg=null;
  window.fetch=async(p,a)=>p==='/api/config'&&!a?.method?{json:async()=>({durum:'ok',veri:g})}:f1(p,a);
  const k=konumDenetle();await bek(400);
  esit('name-only city: picker asks',q('#sor2 h1')?.textContent,'Yeşilova: which one?');
  q('#sor2 select').value='0';q('#sor2 form').requestSubmit();await k;
  esit('name-only city: saved with coordinates',cfg?.konum,{ad:'Yeşilova',bolge:'Burdur',ulke:'Türkiye',lat:37.5,lon:29.75});
  g={sehir:'Yeşilova',konum:true};cfg=null;await konumDenetle();
  esit('saved place: nothing asked',[cfg,q('#sor2').open],[null,false]);
  window.fetch=f1;}

// the sun theme changes in the first tick of a new minute, even when second 0 was skipped
{const tg=window.temaGecis,t0=S.tema;let n=0;window.temaGecis=()=>n++;
  S.tema='gunes';const hedef=temaHedef(),p0=K.dataset.palet;K.dataset.palet=hedef+'x';
  sonDk=Math.floor(Date.now()/6e4)-1;tick();
  esit('sun theme: switched on a new minute',n,1);
  tick();esit('sun theme: once a minute',n,1);
  window.temaGecis=tg;S.tema=t0;K.dataset.palet=p0;}

// while locked the city question waits for the PIN (it would open on top of the lock screen)
{const f1=window.fetch;let n=0;window.fetch=async(p,a)=>{if(p==='/api/config')n++;return f1(p,a)};
  kilitli=true;await konumDenetle();kilitli=false;window.fetch=f1;
  esit('locked: city not asked',[n,q('#sor2').open],[0,false]);}
