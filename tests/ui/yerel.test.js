// dil: en
// 0.6 locale conventions in en-US: 12-hour clock, Sunday-first weeks, °F/mph, holiday calendar by region; each one can be changed
const {esit,dogru,q,bek}=RT;
const f0=window.fetch,istek=[];
window.fetch=async(p,a)=>{
  if(typeof p==='string'&&(p.startsWith('/api/weather')||p.startsWith('/api/calendar')))istek.push(p+' '+(a?.body||''));
  if(p==='/api/weather?birim=f'){const r=await (await f0('/api/weather')).json();r.veri=JSON.parse(JSON.stringify(r.veri));r.veri.birim='f';r.veri.current.temperature_2m=63;r.veri.current.wind_speed_10m=6;r.veri.current.precipitation=0.01;return{json:async()=>r}}
  if(p==='/api/state'&&a?.method==='POST')return{json:async()=>({durum:'ok'})};
  return f0(p,a)};
const d=new Date(2026,10,4,18,5);

// time format
esit('12 h by default in en-US',[OTO12,saat(d),saatS('07:30'),saat24(d)],[true,'6:05 PM','7:30 AM','18:05']);
S.ayar.saatBicim='h24';esit('24 h when chosen',[saat(d),saatS('07:30')],['18:05','07:30']);
S.ayar.saatBicim='h12';esit('12 h when chosen',saat(new Date(2026,0,1,0,7)),'12:07 AM');
esit('second clock follows the format',dilimSaat('UTC',new Date(Date.UTC(2026,0,1,15,4))),'03:04 PM');
S.ayar.saatBicim='h24';esit('second clock in 24 h',dilimSaat('UTC',new Date(Date.UTC(2026,0,1,15,4))),'15:04');
delete S.ayar.saatBicim;
esit('datetime-local stays 24 h',yerel(d),'2026-11-04T18:05');
// the second clock is not shown twice when it is the computer's own zone
const yerelTz=Intl.DateTimeFormat().resolvedOptions().timeZone;
if(SAAT2[yerelTz]!==undefined){S.ayar.saat2=yerelTz;ustSon='';ustCiz();dogru('own zone not repeated',q('#saat2').hidden||!q('#saat2').textContent.startsWith(SAAT2[yerelTz]+' '))}

// first day of the week: 1 Nov 2026 is a Sunday
const ilk=()=>[...q('#grid').children].slice(0,7).map(x=>x.textContent);
const bosluk=()=>q('#grid').querySelectorAll(':scope>span').length;
ref=new Date(2026,10,1);gorunum='ay';izgara();
esit('Sunday first in en-US',[OTO_HB,ilk()[0],ilk()[6],bosluk()],[0,'Sun','Sat',0]);
S.ayar.haftaBasi='pzt';izgara();
esit('Monday first when chosen',[ilk()[0],bosluk()],['Mon',6]);
gorunum='hafta';ref=new Date(2026,10,4);
esit('week view starts on Monday',gunStr(aralik().bas),'2026-11-02');
S.ayar.haftaBasi='paz';esit('week view starts on Sunday',gunStr(aralik().bas),'2026-11-01');
gorunum='ay';ref=new Date();ref.setHours(0,0,0,0);delete S.ayar.haftaBasi;

// units: °F, mph, inch in en-US; the request asks for them and the labels follow the data
istek.length=0;await hava();
dogru('weather asked in °F',istek.some(x=>x.startsWith('/api/weather?birim=f ')));
dogru('mph shown',q('#hava .now').textContent.includes('6 mph'));
dogru('inch shown',q('#havaEk')?.textContent.includes('0.01 in')??true);
S.ayar.birim='c';istek.length=0;await hava();
dogru('metric when chosen',istek.some(x=>x.startsWith('/api/weather '))&&q('#hava .now').textContent.includes('km/h'));
delete S.ayar.birim;

// public holidays: by region, a chosen country, or off
esit('holidays by region',tatilId(),'en.usa#holiday@group.v.calendar.google.com');
S.ayar.tatil='turkish';esit('chosen country',tatilId(),'en.turkish#holiday@group.v.calendar.google.com');
S.ayar.tatil='yok';esit('off',tatilId(),'yok');
istek.length=0;await takvimAl(new Date(2026,10,1),new Date(2026,11,1),true);
dogru('calendar request carries the choice',istek.some(x=>x.includes('"tatil":"yok"')));
S.ayar.tatil='bilinmeyen';esit('unknown value falls back',tatilId(),'en.usa#holiday@group.v.calendar.google.com');
delete S.ayar.tatil;

// the new settings exist in the card dialogs
kartAyar('saat');await bek(100);dogru('clock: time format setting',!!q('#sor [data-a="saatBicim"]'));
kartAyar('takvim');await bek(100);dogru('calendar: week start and holidays',!!q('#sor [data-a="haftaBasi"]')&&q('#sor select[data-a="tatil"]')?.options.length===19);
kartAyar('hava');await bek(100);dogru('weather: units',!!q('#sor [data-a="birim"]'));
