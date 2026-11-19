// dil: tr
// Next 7 days lists only days with events; an empty week shows no block
const {esit}=RT;
const bu=gunStr(new Date());
ileri=[{start:{date:bu},end:{date:gunStr(new Date(Date.now()+864e5))},summary:'Toplantı'}];takvimEk();
esit('only the day with an event',[...document.querySelectorAll('#takvimEk .yedi>div')].map(d=>d.textContent),['BugünToplantı']);
ileri=[{start:{date:'2000-01-01'},end:{date:'2000-01-02'},summary:'Eski'}];takvimEk();
esit('empty week: no block',document.querySelector('#takvimEk')?.innerHTML,'');
// Quick links show the site icon when one is known, otherwise the letter
S.links=[{a:'GitHub',u:'https://github.com/x'},{a:'Kitap',u:'https://ornek.test'}];FAV={'github.com':{v:'data:image/png;base64,iVBORw0KGgo=',t:Date.now()}};linkCiz();
esit('icon and letter',[...document.querySelectorAll('#linkler [data-l]:not([data-l="+"])')].map(b=>[!!b.querySelector('img'),b.querySelector('span').textContent]),[[true,'G'],[false,'K']]);
await RT.bek(100);RT.dogru('unknown host was asked once and remembered as failed',FAV['ornek.test']?.v==='');
// A timed event ending at 00:00 stays on its own day; an all-day event still ends the day before its end date
esit('ends at midnight',[gunleri({start:{dateTime:'2026-11-20T22:00:00'},end:{dateTime:'2026-11-21T00:00:00'}}),gunleri({start:{date:'2026-11-20'},end:{date:'2026-11-22'}})],[['2026-11-20'],['2026-11-20','2026-11-21']]);
// Currency: silver and world rates are asked for only when the card shows them; every request carries the window key
{const f=window.fetch,yol=[];window.fetch=(p,o)=>{yol.push([p,o?.headers?.['X-Ryocs']]);return f(p,o)};kartAc('kur',true);S.ayar.kurTaban='';
 const xml='<T Tarih="19.11.2026"><Currency CurrencyCode="USD"><Unit>1</Unit><ForexSelling>40</ForexSelling></Currency><Currency CurrencyCode="EUR"><Unit>1</Unit><ForexSelling>46</ForexSelling></Currency></T>';
 kurV=null;await kur(true); // nothing fetched yet: world rates too
 kurV=kurAyir(xml,3000,null,{PLN:4});S.ayar.kurListe='USD, EUR';S.ayar.gumus=false;await kur(true); // both in the TCMB feed
 S.ayar.kurListe='USD, BGN';S.ayar.gumus=true;await kur(true); // BGN is not in the feed (TCMB dropped it): world rates
 S.ayar.kurListe='PLN';S.ayar.gumus=false;await kur(true);window.fetch=f; // PLN came as a cross rate last time: still needed
 esit('only what is shown',yol.filter(([p])=>p.startsWith('/api/kur')).map(([p])=>p).slice(-4),['/api/kur?ag=0&dunya=1','/api/kur?ag=0&dunya=0','/api/kur?ag=1&dunya=1','/api/kur?ag=0&dunya=1']);
 esit('key header sent',yol.every(([,h])=>h===''),true);} // no key in the test page (no Rust window), but the header is always set
// The focus timer is kept in localStorage; a round that ended while RYOCS was closed counts on its own day, without starting the next one
{const g=gunStr(new Date(Date.now()-864e5)),once=S.odak[g]||0;O={mod:'calisma',kalan:0,bitis:Date.now()-864e5};S.ayar.odakOto=true;odakCiz();
 esit('round counted on the day it ended, next one not started',[S.odak[g]-once,O.mod,O.bitis],[1,'mola',0]);
 esit('stored',JSON.parse(localStorage.odakO),{mod:'mola',kalan:sure('mola'),bitis:0});}
