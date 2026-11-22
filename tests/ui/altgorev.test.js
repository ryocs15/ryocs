// dil: tr
// Subtasks: added from the task menu (comma separated), ticked, counter, search finds the task by them, a repeating task opens them
// again at the next date (undo brings them back), deleted from their own menu, damaged ones dropped on load
const {esit,dogru,q,bek}=RT;
for(const x of document.querySelectorAll('dialog[open]'))x.close();
sekme='gorev';suz='tumu';$('#ara').value='';
S.todos=[{t:'Taşınma'},{t:'Haftalık rapor',due:yerel(new Date(Date.now()+36e5)),tek:'hafta'}];ciz();
const menuBas=t=>[...q('#menu').querySelectorAll('button')].find(b=>b.textContent===t).click();
gorevMenu(10,10,0);menuBas('Alt görev ekle…');await bek(60);
q('#sor input[name=a0]').value='Kutular, Nakliye ,  , Adres değişikliği';q('#sor form').requestSubmit();await bek(60);
esit('three subtasks, blanks dropped',S.todos[0].alt,[{t:'Kutular',d:false},{t:'Nakliye',d:false},{t:'Adres değişikliği',d:false}]);
esit('shown under the task',[...q('#todos').querySelectorAll('[data-ar="0"] .altO span')].map(s=>s.textContent),['Kutular','Nakliye','Adres değişikliği']);
esit('counter',q('#todos [data-r="0"] .altSay').textContent,'0/3');
q('#todos [data-ai="0:1"]').click();
esit('ticked',[S.todos[0].alt[1].d,q('#todos [data-r="0"] .altSay').textContent,q('#todos [data-ai="0:1"]').getAttribute('aria-checked')],[true,'1/3','true']);
dogru('task not completed by its subtasks',!S.todos[0].d);
// search by a subtask's text
$('#ara').value='nakliye';ciz();
esit('found by subtask',[...q('#todos').querySelectorAll('.row>span')].map(s=>s.textContent),['Taşınma']);
$('#ara').value='';ciz();
const g=araHesapla('nakliye').find(x=>x[0]==='Görevler')?.[1]?.[0];
esit('Ctrl+F: task found by its subtask',[g?.ad,g?.alt.replace(/<[^>]+>/g,'')],['Taşınma','Nakliye']);
// repeating task: next date opens the subtasks again; undo brings the ticks back
S.todos[1].alt=[{t:'Veri',d:true}];ciz();
isaretle(1);
esit('repeat: subtasks open again',S.todos[1].alt[0].d,false);
q('#tost button')?.click();await bek(20);
esit('undo: tick back',S.todos[1].alt[0].d,true);
// delete from the subtask's menu (last one removes the list)
q('#todos [data-ao="1:0"]').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:10,clientY:10}));await bek(20);
menuBas('Sil');await bek(400);
esit('last subtask deleted: no list left',[S.todos[1].alt,!!q('#todos [data-ar="1"]')],[undefined,false]);
// damaged data on load
esit('damaged subtasks dropped',normal({todos:[{t:'a',alt:[{t:'x',d:1},null,{d:true},'y']},{t:'b',alt:'bozuk'},{t:'c'}]}).todos,[{t:'a',alt:[{t:'x',d:true}]},{t:'b'},{t:'c'}]);
