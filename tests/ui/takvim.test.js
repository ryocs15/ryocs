// dil: tr
// Calendar: reminder lead time, join meeting / open in Maps in the details, arrow keys in the month grid
const {esit,dogru,q,bek}=RT;
const acilan=[],bild=[];const f0=window.fetch;
window.fetch=async(p,a)=>{
  if(p==='/api/open'){acilan.push(JSON.parse(a.body).url);return{json:async()=>({durum:'ok'})}}
  if(p==='/api/notify'){bild.push(JSON.parse(a.body));return{json:async()=>({durum:'ok'})}}
  return f0(p,a)};
for(const x of document.querySelectorAll('dialog[open]'))x.close();
S.ayar.rahatsiz=false;S.ayar.etk=true;await bek(300);
// reminder lead time: an event in 20 minutes waits with 10, is announced with 30
const e20={id:'r1',summary:'Ders',start:{dateTime:new Date(Date.now()+20*6e4).toISOString()},_t:'primary'};
ileri=[e20];S.ayar.etkDk=10;hatirlat();esit('10 min: not yet',bild.filter(b=>b.baslik==='Yaklaşan etkinlik').length,0);
S.ayar.etkDk=30;hatirlat();esit('30 min: announced',bild.filter(b=>b.baslik==='Yaklaşan etkinlik').map(b=>b.metin.split(' · ')[0]),['Ders']);
S.ayar.etkDk=99;esit('a bad value: 10',etkDk(),10);
// details: meeting link and place
evler=[{id:'m',summary:'Toplantı',start:{dateTime:new Date().toISOString()},location:'Kızılay, Ankara',hangoutLink:'https://meet.google.com/abc-defg-hij',_t:'primary'},
  {id:'x',summary:'Kötü bağlantı',start:{date:gunStr(new Date())},conferenceData:{entryPoints:[{entryPointType:'video',uri:'javascript:alert(1)'}]},_t:'primary'}];
detay(0);await bek(50);q('#eMeet').click();q('#eHarita').click();
esit('meeting and map opened',acilan,['https://meet.google.com/abc-defg-hij','https://www.google.com/maps/search/?api=1&query=K%C4%B1z%C4%B1lay%2C%20Ankara']);
detay(1);await bek(50);esit('not https: no join button, no map without a place',[!!q('#eMeet'),!!q('#eHarita')],[false,false]);
for(const x of document.querySelectorAll('dialog[open]'))x.close();
// arrow keys: next day, a week down, past the end of the month the next month opens
gorunum='ay';ayGit(0);await bek(300);
const bugun=new Date(),son=new Date(bugun.getFullYear(),bugun.getMonth()+1,0),t=gunStr(son);
q(`#grid [data-t="${t}"]`).focus();q(`#grid [data-t="${t}"]`).dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowRight',bubbles:true}));await bek(400);
const ilk=gunStr(new Date(bugun.getFullYear(),bugun.getMonth()+1,1));
esit('past the month end: next month, first day focused',document.activeElement?.dataset.t,ilk);
document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true}));await bek(100);
esit('down: a week later',document.activeElement?.dataset.t,gunStr(new Date(bugun.getFullYear(),bugun.getMonth()+1,8)));
