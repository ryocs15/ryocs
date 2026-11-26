// dil: tr
// 1.1.6 weather: current icon, rain timing, yesterday, air quality, a day's hours, morning alert, evening note
const {esit,dogru,q,bek}=RT;
const notlar=[];const f0=window.fetch;
window.fetch=async(p,a)=>{if(p==='/api/notify'){notlar.push(JSON.parse(a.body));return{json:async()=>({durum:'ok'})}}return f0(p,a)};
await bek(300);
const kopya=()=>JSON.parse(JSON.stringify(havaV)),saat=h=>{const d=new Date();d.setMinutes(0,0,0);d.setHours(d.getHours()+h);return gunStr(d)+'T'+iki(d.getHours())+':00'};
// 48 hours from now: dry, then rain from +3 h
const ciz=w=>{havaV=w;havaSehir='';havaSec=null;S.ayar.havaGun=false;havaCiz()};
const w=kopya();w.current.time=saat(0);w.current.precipitation=0;w.current.temperature_2m=12;w.current.is_day=0;w.current.weather_code=0;w.dun=16;
w.hourly={time:[],temperature_2m:[],precipitation_probability:[],weather_code:[],is_day:[]};
const g0=new Date();g0.setHours(0,0,0,0);
for(let h=0;h<24*7;h++){const d=new Date(+g0+h*36e5);w.hourly.time.push(gunStr(d)+'T'+iki(d.getHours())+':00');w.hourly.temperature_2m.push(h);
  const ileri=(d-Date.now())/36e5;w.hourly.precipitation_probability.push(ileri>=2.5&&ileri<6?80:0);w.hourly.weather_code.push(ileri>=2.5&&ileri<6?63:1);w.hourly.is_day.push(1)}
w.kalite={european_aqi:35,pm2_5:12.4,grass_pollen:22,birch_pollen:3};
ciz(w);
dogru('current icon: moon at night',!!q('#hava .now>svg.hi-ay'));
const not=q('#hava .hvNot')?.textContent||'';
dogru('rain starts',/^Yağmur \S+ civarı başlıyor/.test(not));
dogru('yesterday',not.endsWith('Dünden 4° daha soğuk'));
// raining now: when it stops
w.current.precipitation=1.2;w.hourly.precipitation_probability=w.hourly.precipitation_probability.map((v,k)=>new Date(w.hourly.time[k])<new Date(Date.now()+2.5*36e5)?90:10);ciz(w);
dogru('rain stops',/^Yağış \S+ civarı diniyor/.test(q('#hava .hvNot').textContent));
{const pp=w.hourly.precipitation_probability,wc=w.hourly.weather_code,i=w.hourly.time.findIndex(x=>x.slice(0,13)>=yerSaat(w));
 w.current.precipitation=0;w.hourly.precipitation_probability=pp.map((v,k)=>k===i?70:0);w.hourly.weather_code=wc.map((v,k)=>k===i?61:1);ciz(w);
 dogru('rain within this hour: soon',q('#hava .hvNot').textContent.startsWith('Yağmur yakında başlıyor'));
 w.hourly.precipitation_probability=pp;w.hourly.weather_code=wc;w.current.precipitation=1.2;}
w.dun=12;ciz(w);dogru('same as yesterday',q('#hava .hvNot').textContent.endsWith('Dünle aynı'));
delete w.dun;w.current.precipitation=0;w.hourly.precipitation_probability=w.hourly.precipitation_probability.map(()=>0);ciz(w);
esit('nothing to say: no line',q('#hava .hvNot'),null);
// air quality and pollen (in the details block)
const ek=havaEk(w);
dogru('air quality',ek.includes('<small>Hava kalitesi</small><b>Makul</b>')&&ek.includes('PM2.5 12 µg/m³ · polen: çimen (orta)'));
dogru('pollen shown when moderate',ek.includes('<small>Polen</small><b>orta</b>'));
dogru('no data: no block',!havaEk({...w,kalite:null}).includes('Hava kalitesi'));
esit('index names',[aqiAd(10),aqiAd(55),aqiAd(120)],['İyi','Orta','Aşırı kötü']);
// a day's hours: the 7-day row and the 3-day strip open it, the same day again closes it
for(const x of document.querySelectorAll('dialog[open]'))x.close(); // the welcome dialog makes the page inert
q('#hava').style.minHeight='1100px';ciz(w);await bek(300);
const satir=q('#hava .hv7 [data-g7="2"]');dogru('7-day row is a button',satir?.getAttribute('role')==='button');
satir.click();await bek(50);
const gs=q('#hava .saatlik.gunSaat');
esit('day hours',[!!gs,gs?.querySelector('small').textContent,gs?.querySelectorAll(':scope>div').length],[true,new Date(w.daily.time[2]+'T00:00').toLocaleDateString(LOC,{weekday:'short'}),6]);
esit('focus stays on the row, it says pressed',[document.activeElement?.dataset.g7,document.activeElement?.closest('.hv7')!=null,q('#hava .hv7 [data-g7="2"]').getAttribute('aria-pressed'),q('#hava .hv7 [data-g7="3"]').getAttribute('aria-pressed')],['2',true,'true','false']);
esit('buttons: none pressed, row marked',[[...q('#hava').querySelectorAll('[data-hg]')].map(b=>b.getAttribute('aria-pressed')),q('#hava .hv7 [data-g7="2"]').classList.contains('sec')],[['false','false'],true]);
q('#hava .hv7 [data-g7="2"]').click();await bek(50);
dogru('same day again: back',!q('#hava .gunSaat')&&!!q('#hava .saatlik'));
q('#hava .hv7 [data-g7="3"]').dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true}));await bek(50);
dogru('keyboard opens it',!!q('#hava .gunSaat'));
q('#hava [data-hg="1"]').click();await bek(50);
dogru('3 days: strip back',!!q('#hava .saatlik.gunluk')&&!q('#hava .gunSaat'));
q('#hava .saatlik.gunluk [data-g7="1"]').click();await bek(50);
dogru('3-day strip opens a day',!!q('#hava .gunSaat'));
esit('strip gone: focus on "3 days"',document.activeElement?.dataset.hg,'1');
havaSec={k:'baska',g:w.daily.time[1]};ciz(w);havaSec={k:'baska',g:w.daily.time[1]};havaCiz();
dogru('another city: dropped',!q('#hava .gunSaat'));
// morning alert and evening note (the clock is moved for the test)
const gh=Date.prototype.getHours,saatYap=h=>Date.prototype.getHours=function(){return h};
const u=kopya();u.daily.time[0]=gunStr(new Date());u.daily.weather_code[0]=95;u.daily.wind_gusts_10m_max=[72];u.daily.temperature_2m_max[0]=36;u.daily.precipitation_probability_max[0]=10;u.daily.temperature_2m_min[0]=8;u.kalite={european_aqi:70};u.birim='c';
havaV=u;S.ayar.havaUyari=true;S.havaUyari='';saatYap(8);notlar.length=0;havaUyar();
esit('morning alert',notlar[0]?.metin,'Bugün gök gürültülü fırtına, kuvvetli rüzgâr (72 km/sa), aşırı sıcak (en yüksek 36°), hava kalitesi kötü.');
havaUyar();esit('once a day',notlar.length,1);
u.daily.weather_code[1]=61;u.daily.temperature_2m_max[1]=14.4;u.daily.temperature_2m_min[1]=7.6;u.daily.precipitation_probability_max[1]=70;
S.ayar.havaAksam=true;S.havaAksam='';saatYap(17);havaAksam();esit('evening: not before 18',notlar.length,1);
saatYap(19);havaAksam();havaAksam();
esit('evening note once',[notlar.length,notlar[1]?.baslik,notlar[1]?.metin],[2,'Yarın · '+u.sehir,'Hafif yağmur, 14° / 8°, yağış %70.']);
S.havaAksam='';S.ayar.havaAksam=false;havaAksam();esit('evening note off',notlar.length,2);
Date.prototype.getHours=gh;

// a block ending at the bottom of the card extra keeps its shadow and focus ring: the extra clips 14px outside its box, not at the edge
{const c=getComputedStyle(q('[data-kart=hava] .ek'));esit('extra keeps shadows of its last block',[c.overflowY,c.overflowClipMargin],['clip','14px'])}
