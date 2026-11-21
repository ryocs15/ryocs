// dil: tr
// Tasks: a date in the quick-add text, snooze, "complete all" counted for the chart
const {esit,dogru,q,bek}=RT;
for(const x of document.querySelectorAll('dialog[open]'))x.close();
// Thursday 2026-11-19, 10:30
const an=new Date(2026,10,19,10,30);
const c=m=>{const r=gorevCoz(m,an);return[r.t,r.due||null]};
esit('tomorrow with a time',c('Dişçi yarın 15:00'),['Dişçi','2026-11-20T15:00']);
esit('weekday alone: 09:00',c('Rapor cuma'),['Rapor','2026-11-20T09:00']);
esit('time alone, later today',c('Annemi ara saat 18.30'),['Annemi ara','2026-11-19T18:30']);
esit('a price with a dot: not a time',c('Fatura öde 12.50'),['Fatura öde 12.50',null]);
esit('Turkish verbs are not days',[c('Arabayı sat'),c('Raporu sun'),c('Kuşu sal')],[['Arabayı sat',null],['Raporu sun',null],['Kuşu sal',null]]);
esit('unambiguous short forms still work',c('Spor pzt 07:00'),['Spor','2026-11-23T07:00']);
esit('time alone, passed: tomorrow',c('Koşu saat 07:00'),['Koşu','2026-11-20T07:00']);
esit('same weekday, time passed: next week',c('Toplantı perşembe 09:00'),['Toplantı','2026-11-26T09:00']);
esit('English',c('Gym tomorrow 7:15'),['Gym','2026-11-20T07:15']);
esit('nothing found: unchanged',c('Pazartesi raporu'),['Pazartesi raporu',null]);
esit('only a date word: kept as text',c('yarın'),['yarın',null]);
esit('pazar is not pazartesi',c('Pazar kahvaltısı pazartesi'),['Pazar kahvaltısı','2026-11-23T09:00']);
// quick add through the form
const n0=S.todos.length;q('#yeni').value='Fatura öde yarın 12:00';q('#tform').requestSubmit();await bek(30);
const t=S.todos.at(-1);esit('added with a reminder',[S.todos.length-n0,t.t,!!t.due],[1,'Fatura öde',true]);
// snooze
t.due='2026-01-01T08:00';t.n=true;ertele(t,'yarin');
const y=new Date();y.setDate(y.getDate()+1);
esit('snoozed to tomorrow, same time, reminds again',[t.due,t.n],[gunStr(y)+'T08:00',undefined]);
const once=Date.now();ertele(t,'saat');dogru('an hour later',Math.abs(new Date(t.due)-once-36e5)<6e4);
// complete all: counted for the 7-day chart
S.todos=[{t:'a',d:false},{t:'b',d:true},{t:'c',d:false},{t:'d',d:false,tek:'gun',due:'2030-01-01T09:00'}];const g=gunStr(new Date()),b0=S.biten[g]||0;
sekme='gorev';ciz();q('#tmenu').click();await bek(30);[...q('#menu').querySelectorAll('button')].find(b=>b.textContent==='Tümünü tamamla').click();
esit('complete all: 2 more in the chart, repeating one untouched',[(S.biten[g]||0)-b0,S.todos.map(t=>t.d)],[2,[true,true,true,false]]);
