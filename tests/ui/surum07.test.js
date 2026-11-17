// dil: en
// sure: 25000
// 0.7–0.8: reminders from the next 6 months, editing events, focus history, first-run tips, settings search, shortcut choice
const {esit,dogru,q,bek}=RT;
const f0=window.fetch,bildirim=[],duzenle=[],kisayol=[];
window.fetch=async(p,a)=>{
  if(p==='/api/notify'){bildirim.push(JSON.parse(a.body));return{json:async()=>({durum:'ok'})}}
  if(p==='/api/calendar/edit'){duzenle.push(JSON.parse(a.body));return{json:async()=>({durum:'ok'})}}
  if(p==='/api/kisayol'){const b=JSON.parse(a.body);kisayol.push(b);return{json:async()=>({durum:'ok',veri:b.i==null?null:['Win+Ctrl+R','Win+Alt+Y','Ctrl+Alt+Shift+R','Ctrl+Alt+Space'][b.i]})}}
  if(p==='/api/config'&&!a?.method)return{json:async()=>({durum:'ok',veri:{sehir:'Köyceğiz',kisayol:'Win+Alt+Y',kisayollar:['Win+Ctrl+R','Win+Alt+Y','Ctrl+Alt+Shift+R','Ctrl+Alt+Space'],google:'kimlik_yok',spotify:'kimlik_yok',surum:'0.7.0'}})};
  if(p==='/api/state'&&a?.method==='POST')return{json:async()=>({durum:'ok'})};
  return f0(p,a)};
document.querySelectorAll('dialog[open]').forEach(d=>d.close());

// 0.7.1 an event in 5 minutes is reminded even when the calendar shows another month
ileri=[{id:'r1',summary:'Dentist',start:{dateTime:new Date(Date.now()+5*6e4).toISOString()},end:{dateTime:new Date(Date.now()+65*6e4).toISOString()},_t:'primary'}];
ref=new Date(2030,0,1);bildirim.length=0;hatirlat();await bek(50);
dogru('reminder from the 6-month list',bildirim.some(b=>b.baslik==='Upcoming event'&&b.metin.startsWith('Dentist')));
ref=new Date();ref.setHours(0,0,0,0);ileri=[];

// 0.8.1 editing events: the duration is kept, all-day spans keep their length, a series changes its title only
const ac=async(e,set)=>{const s=etkDuzenle(e);await bek(150);for(const [k,v] of Object.entries(set))q(`#sor [name=${k}]`).value=v;q('#sor form').requestSubmit();await s;await bek(50);return duzenle.at(-1)};
const zamanli={id:'e1',_t:'primary',summary:'Meeting',start:{dateTime:'2026-11-17T18:00:00'},end:{dateTime:'2026-11-17T19:30:00'}};
let g=await ac(zamanli,{a0:'Meeting 2',a1:'2026-11-20'});
const b=new Date(2026,10,20,18,0);
esit('timed event moved, 90 min kept',g,{takvim:'primary',id:'e1',summary:'Meeting 2',start:{dateTime:offTarih(b),date:null},end:{dateTime:offTarih(new Date(b.getTime()+90*6e4)),date:null}});
g=await ac({id:'e2',_t:'primary',summary:'Trip',start:{date:'2026-11-17'},end:{date:'2026-11-20'}},{a1:'2026-12-01'});
esit('3-day event keeps 3 days',[g.start,g.end],[{date:'2026-12-01',dateTime:null},{date:'2026-12-04',dateTime:null}]);
g=await ac(zamanli,{a2:''});
esit('timed → all day',[g.start,g.end],[{date:'2026-11-17',dateTime:null},{date:'2026-11-18',dateTime:null}]);
g=await ac({id:'e3_20261117',recurringEventId:'e3',_t:'primary',summary:'Gym',start:{dateTime:'2026-11-17T07:00:00'},end:{dateTime:'2026-11-17T08:00:00'}},{a0:'Gym (new)',a3:'seri'});
esit('series: title only, series id',g,{takvim:'primary',id:'e3',summary:'Gym (new)'});

// 0.8.2 focus history by "working on" text
S.odakLog=[];S.ayar.odakNe='Thesis';O={mod:'calisma',kalan:0,bitis:Date.now()-1000};odakCiz();
esit('finished round logged',S.odakLog.map(x=>[x.dk,x.ne]),[[25,'Thesis']]);
S.odakLog.push({g:gunStr(new Date()),dk:50});
dogru('weekly breakdown',odakNeEk().includes('Thesis')&&odakNeEk().includes('25 min')&&odakNeEk().includes('No note'));
S.odakLog=[{g:'2000-01-01',dk:25}];esit('older than a week not shown',odakNeEk(),'');
O={mod:'calisma',kalan:sure('calisma'),bitis:0};

// 0.8.4 tips once
document.querySelectorAll('dialog[open]').forEach(d=>d.close());delete S.ayar.tur;turGoster();await bek(100);
esit('tips shown',[q('#sor h1')?.textContent,q('#sor').querySelectorAll('.tur li').length,S.ayar.tur],['A few tips',4,1]);
q('#sor [data-kapat]').click();await bek(300);turGoster();await bek(100);
dogru('tips only once',!q('#sor').open);

// 0.8.5 settings search
ayarAc();await bek(300);
const ara=v=>{q('#ayAra').value=v;q('#ayAra').dispatchEvent(new Event('input'))};
const gorunen=()=>[...document.querySelectorAll('.ayBol:not([hidden]) .ar:not(.araGizli)')].map(r=>r.textContent.trim().slice(0,19));
ara('backup');
esit('search: matching rows from every category',[q('#ayBas').textContent,gorunen()],['Search results',['Layout backupscard ','Restore from backup','Backup folderOpen f']]);
RT.dogru('search: section headings give way to category names',[...document.querySelectorAll('.ayBol.ayBulundu>h3')].every(h=>!h.offsetParent)&&getComputedStyle(q('.ayBol.ayBulundu:not([hidden])'),'::before').content.includes('Cards'));
ara('zzqx');esit('search: nothing found',[q('#ayBas').textContent,gorunen()],['No results',[]]);
q('.ayNav [data-b="genel"]').click();await bek(50);
esit('category click ends the search',[q('#ayAra').value,!!document.querySelector('#ayarlar .araGizli'),q('#ayBas').textContent.trim()],['',false,'App']);

// 0.8.7 shortcut choice
await ayarCiz();await bek(100);
esit('shortcut list',[[...q('#aKbd').options].map(o=>o.textContent),q('#aKbd').value],[['Win+Ctrl+R','Win+Alt+Y','Ctrl+Alt+Shift+R','Ctrl+Alt+Space','Off'],'1']);
q('#aKbd').value='3';q('#aKbd').dispatchEvent(new Event('change'));await bek(100);
esit('shortcut changed',[kisayol.at(-1),S.ayar.kisayol],[{i:3},3]);
q('#aKbd').value='yok';q('#aKbd').dispatchEvent(new Event('change'));await bek(100);
esit('shortcut off',[kisayol.at(-1),S.ayar.kisayol],[{i:null},'yok']);
