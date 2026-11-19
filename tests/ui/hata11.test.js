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
{const f=window.fetch,yol=[];window.fetch=(p,o)=>{yol.push([p,o?.headers?.['X-Ryocs']]);return f(p,o)};kartAc('kur',true);
 S.ayar.kurTaban='';S.ayar.kurListe='USD, EUR';S.ayar.gumus=false;await kur(true);S.ayar.kurListe='USD, PLN';S.ayar.gumus=true;await kur(true);window.fetch=f;
 esit('only what is shown',yol.filter(([p])=>p.startsWith('/api/kur')).map(([p])=>p).slice(-2),['/api/kur?ag=0&dunya=0','/api/kur?ag=1&dunya=1']);
 esit('key header sent',yol.every(([,h])=>h===''),true);} // no key in the test page (no Rust window), but the header is always set
