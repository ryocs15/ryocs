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
