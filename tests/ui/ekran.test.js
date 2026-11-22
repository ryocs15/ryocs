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
delete S.ayar.ekranSinir;delete S.ayar.ekranGizli;
dogru('closed by default for an older save',!kartAcik('ekran'));
kartAc('ekran',true);await bek(200);
esit('opening the card starts measuring',giden.at(-1),{acik:true});
const satir=()=>[...q('#ekran').querySelectorAll('[data-e] .sisS')].map(s=>s.textContent);
esit('total today',q('#ekran .ekTop b').textContent,'5 sa 41 dk');
esit('top six, longest first, name escaped',satir(),['Microsoft Edge3 sa','Visual Studio Code2 sa 10 dk','Discord30 dk','<b>x1 dk','A<1 dk','B<1 dk']);
esit('seven day bars, today highlighted',[q('#ekran .ekHafta').children.length,q('#ekran .ekHafta i:last-child').classList.contains('bu')],[7,true]);
dogru('chart has a text summary',q('#ekran .ekHafta').getAttribute('aria-label').startsWith('Son 7 gün: '));

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

// rename: the row shows the new name; two programs with the same name share one row and their time adds up
ekranSinirYaz('Discord',10);S.ayar.ekranGizli=['A'];
ekranAdYaz('Discord','Sohbet');
esit('renamed row',satir()[2],'Sohbet30 dk / 10 dk');
esit('limit follows the new name',A('ekranSinir'),{Sohbet:10});
ekranAdYaz('Visual Studio Code','Sohbet');
esit('same name merges',satir()[1],'Sohbet2 sa 40 dk / 10 dk');
esit('total unchanged',q('#ekran .ekTop b').textContent,'5 sa 41 dk');
esit('notification uses the new name',bildirim.at(-1).metin.startsWith('Sohbet: bugünkü 10 dk'),true);
ekranAdYaz('A','Gizli');
esit('hidden follows the new name',A('ekranGizli'),['Gizli']);
// back to the own names: both rows return, each keeps the limit
ekranAdYaz('Sohbet','');
esit('own names back',[A('ekranAd'),satir().slice(1,3)],[{A:'Gizli'},['Visual Studio Code2 sa 10 dk / 10 dk','Discord30 dk / 10 dk']]);
ekranAdYaz('Gizli','  ');
esit('empty name goes back',[A('ekranAd'),A('ekranGizli')],[{},['A']]);
delete S.ayar.ekranSinir;delete S.ayar.ekranGizli;ekranCiz();

// a broken answer does not break the card
ekranCiz({durum:'hata'});
esit('error text',q('#ekran').textContent,'Ekran süresi okunamadı.');

// delete measured data
await ekranAyar(true);
esit('delete sent and card empty',[giden.at(-1),q('#ekran .ekTop b').textContent,!!q('#ekran p.m')],[{acik:true,sil:true},'0 dk',true]);

// closing the card stops measuring
kartAc('ekran',false);await bek(200);
esit('closing stops',giden.at(-1),{acik:false});
