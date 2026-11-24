// dil: tr
// Screen time card: Rust measures only while the card is open; top programs, hidden ones, a daily limit notifies once a day, delete
const {esit,dogru,q,bek}=RT;
const f0=window.fetch,giden=[],bildirim=[];
const bugun=gunStr(new Date()),dun=gunStr(new Date(Date.now()-864e5));
let gunler={[bugun]:{'Microsoft Edge':3*3600,'Visual Studio Code':2*3600+600,Discord:1800,'<b>x':60,A:5,B:5,C:5},[dun]:{'Microsoft Edge':3600}};
window.fetch=async(p,a)=>{
  if(p==='/api/ekran'){const b=JSON.parse(a.body);giden.push(b);if(b.sil)gunler={};return{json:async()=>({durum:'ok',veri:{acik:true,bugun,gunler}})}}
  if(p==='/api/notify'){bildirim.push(JSON.parse(a.body));return{json:async()=>({durum:'ok'})}}
  return f0(p,a)};
try{localStorage.removeItem('ekranUyar')}catch{}
delete S.ayar.ekranSinir;delete S.ayar.ekranGizli;delete S.ayar.ekranToplam;
dogru('closed by default for an older save',!kartAcik('ekran'));
kartAc('ekran',true);await bek(200);
esit('opening the card starts measuring',giden.at(-1),{acik:true});
const satir=()=>[...q('#ekran').querySelectorAll('[data-e] .sisS')].map(s=>s.textContent);
esit('total today',q('#ekran .ekTop b').textContent,'5 sa 41 dk');
esit('top six, longest first, name escaped',satir(),['Microsoft Edge3 sa','Visual Studio Code2 sa 10 dk','Discord30 dk','<b>x1 dk','A<1 dk','B<1 dk']);
esit('seven day bars, today highlighted',[q('#ekran .ekHafta').children.length,q('#ekran .ekHafta button:last-child').classList.contains('bu')],[7,true]);
dogru('bars are named for screen readers',q('#ekran .ekHafta button:last-child').getAttribute('aria-label').endsWith(': 5 sa 41 dk'));
// daily average of the days before today that have time (only yesterday here: 1 h)
esit('average line',q('#ekran .ekOrt').textContent,'Günlük ort. 1 sa · ortalamadan 4 sa 41 dk fazla');
// click yesterday's bar: the list and the total show that day; clicking it again goes back to today
q(`#ekran [data-g="${dun}"]`).click();
esit('a past day',[satir(),q('#ekran .ekTop b').textContent,q(`#ekran [data-g="${dun}"]`).getAttribute('aria-pressed')],[['Microsoft Edge1 sa'],'1 sa','true']);
dogru('day named in the header',q('#ekran .ekTop small').textContent!=='bugün');
q(`#ekran [data-g="${dun}"]`).click();
esit('back to today',[q('#ekran .ekTop small').textContent,satir().length],['bugün',6]);

// hide a program: it leaves the list and the total
S.ayar.ekranGizli=['Microsoft Edge'];ekranCiz();
esit('hidden program',[satir()[0],q('#ekran .ekTop b').textContent],['Visual Studio Code2 sa 10 dk','2 sa 41 dk']);
delete S.ayar.ekranGizli;

// daily limit: passed → warning colour and one notification; the next poll does not repeat it
ekranSinirYaz('Microsoft Edge',120);await bek(50);
esit('limit shown',satir()[0],'Microsoft Edge3 sa / 2 sa');
dogru('passed limit in warning colour',q('#ekran [data-e="Microsoft Edge"] .cizgi').classList.contains('dolu'));
await ekran();await ekran();await bek(50);
esit('one notification',bildirim.filter(b=>b.baslik==='Ekran süresi sınırı').length,1);
dogru('notification names the program',bildirim.at(-1).metin.startsWith('Microsoft Edge: bugünkü 2 sa'));
// changing the limit allows a new notification
ekranSinirYaz('Microsoft Edge',60);await bek(50);
esit('new limit notifies again',bildirim.filter(b=>b.baslik==='Ekran süresi sınırı').length,2);
ekranSinirYaz('Microsoft Edge',0);
esit('limit removed',[A('ekranSinir'),satir()[0]],[{},'Microsoft Edge3 sa']);
// 10 minutes before a limit: one early notification, not repeated
ekranSinirYaz('Microsoft Edge',185);await ekran();await bek(50);
esit('early warning',[bildirim.filter(b=>b.baslik==='Ekran süresi sınırı').length,bildirim.at(-1).metin],[3,'Microsoft Edge: bugünkü sınırına 10 dk kaldı.']);
ekranSinirYaz('Microsoft Edge',0);

// hidden programs come back one by one, or all at once
const menuBas=t=>[...q('#menu').querySelectorAll('button')].find(b=>b.textContent===t).click();
S.ayar.ekranGizli=['A','B'];ekranCiz();
ekranGizliMenu(10,10);menuBas('Göster: A');
esit('one comes back',A('ekranGizli'),['B']);
ekranGizliMenu(10,10);menuBas('Hepsini göster');
esit('all come back',A('ekranGizli'),undefined);

// rename: the row shows the new name; two programs with the same name share one row and their time adds up
ekranSinirYaz('Discord',10);S.ayar.ekranGizli=['A'];
ekranAdYaz('Discord','Sohbet');
esit('renamed row',satir()[2],'Sohbet30 dk / 10 dk');
esit('limit follows the new name',A('ekranSinir'),{Sohbet:10});
await ekran();await bek(50);
dogru('renamed after warning: no second warning',!bildirim.some(b=>b.metin.startsWith('Sohbet:')));
ekranAdYaz('Visual Studio Code','Sohbet');
esit('same name merges',satir()[1],'Sohbet2 sa 40 dk / 10 dk');
esit('total unchanged',q('#ekran .ekTop b').textContent,'5 sa 41 dk');
ekranSinirYaz('Sohbet',10);await bek(50); // setting the limit again allows a new warning, under the new name
esit('notification uses the new name',bildirim.at(-1).metin.startsWith('Sohbet: bugünkü 10 dk'),true);
ekranAdYaz('A','Gizli');
esit('hidden follows the new name',A('ekranGizli'),['Gizli']);
// back to the own names: both rows return, each keeps the limit
ekranAdYaz('Sohbet','');
esit('own names back',[A('ekranAd'),satir().slice(1,3)],[{A:'Gizli'},['Visual Studio Code2 sa 10 dk / 10 dk','Discord30 dk / 10 dk']]);
ekranAdYaz('Gizli','  ');
esit('empty name goes back',[A('ekranAd'),A('ekranGizli')],[{},['A']]);
delete S.ayar.ekranSinir;delete S.ayar.ekranGizli;ekranCiz();

// total daily limit (today 5 sa 41 dk): a bar under the total, orange near the limit, red and a red card edge once passed,
// one notification 10 minutes before and one at the limit
const kart=q('[data-kart=ekran]'),sn=()=>q('#ekran .ekSn');
dogru('no limit, no bar',!sn()&&!kart.classList.contains('ekAsti'));
let n0=bildirim.length;
ekranToplamYaz(500);
esit('under 75%: plain bar and text',[sn().className,sn().querySelector('small').textContent,bildirim.length-n0],['ekSn ','5 sa 41 dk / 8 sa 20 dk',0]);
ekranToplamYaz(450);esit('75%: orange',sn().className,'ekSn yakin');
ekranToplamYaz(360);esit('90%: red, but no card edge before the limit',[sn().className,kart.classList.contains('ekAsti'),bildirim.length-n0],['ekSn asti',false,0]);
ekranToplamYaz(345);
esit('10 minutes before: one warning',[bildirim.length-n0,bildirim.at(-1).metin],[1,'Bugünkü toplam sınırına 10 dk kaldı.']);
ekranToplamYaz(300);
esit('passed: red, card edge, notification',[sn().className,kart.classList.contains('ekAsti'),sn().textContent.endsWith('sınır doldu'),bildirim.at(-1).metin.startsWith('Bugünkü toplam 5 sa sınırını geçtin')],['ekSn asti',true,true,true]);
const n1=bildirim.length;ekranCiz();ekranCiz();
esit('only once a day',bildirim.length,n1);
q(`#ekran [data-g="${dun}"]`).click();
dogru('a past day: no limit bar, the card stays red for today',!sn()&&kart.classList.contains('ekAsti'));
q(`#ekran [data-g="${dun}"]`).click();
// the card menu sets it in hours and minutes, and removes it
q('#ekran').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:10,clientY:10}));await bek(30);
[...q('#menu').querySelectorAll('button')].find(b=>b.textContent==='Günlük toplam sınırı değiştir…').click();await bek(60);
q('#sor input[name=a0]').value='7';q('#sor input[name=a1]').value='30';q('#sor form').requestSubmit();await bek(60);
esit('set from the menu',[A('ekranToplam'),kart.classList.contains('ekAsti'),sn().className],[450,false,'ekSn yakin']);
q('#ekran').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:10,clientY:10}));await bek(30);
[...q('#menu').querySelectorAll('button')].find(b=>b.textContent==='Toplam sınırı kaldır').click();await bek(30);
dogru('removed',A('ekranToplam')===undefined&&!sn());

// a broken answer does not break the card
ekranCiz({durum:'hata'});
esit('error text',q('#ekran').textContent,'Ekran süresi okunamadı.');

// delete measured data
await ekranAyar(true);
esit('delete sent and card empty',[giden.at(-1),q('#ekran .ekTop b').textContent,!!q('#ekran p.m')],[{acik:true,sil:true},'0 dk',true]);

// closing the card stops measuring
kartAc('ekran',false);await bek(200);
esit('closing stops',giden.at(-1),{acik:false});
