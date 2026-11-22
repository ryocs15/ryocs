// dil: tr
// Card height "fit content": no card grows into free space, middle rows are auto, list cards stop at half the window,
// a size set by hand still counts, the default (old saves) fills the space
const {esit,dogru,q,bek}=RT;
for(const x of document.querySelectorAll('dialog[open]'))x.close();
delete S.ayar.kartYuk;delete S.ayar.kartMin;boyutla();
dogru('default: something fills the space',[...document.querySelectorAll('main [data-kart]')].some(e=>e.classList.contains('buyur')));
ayarAc();await bek(80);
q('#aKartYuk [data-v="icerik"]').click();await bek(80);
esit('saved',S.ayar.kartYuk,'icerik');
dogru('switch shows it',q('#aKartYuk [data-v="icerik"]').getAttribute('aria-pressed')==='true');
for(const x of document.querySelectorAll('dialog[open]'))x.close();
esit('no open card grows',[...document.querySelectorAll('main [data-kart]')].filter(e=>!e.hidden&&(e.classList.contains('buyur')||e.style.flexGrow)).map(e=>e.dataset.kart),[]);
dogru('middle rows auto',q('.orta').style.gridTemplateRows.split(' ').every(r=>r==='auto'));
// a long news list stops at half the window
kartAc('haber',true);haberV[haberK()]=[...Array(30)].map((_,i)=>({b:'Haber '+i,l:'https://x/'+i,k:'A',t:''}));haberCiz();await bek(80);
const h=q('[data-kart=haber]').getBoundingClientRect().height;
dogru('list card capped at 50vh',h<=innerHeight/2+1&&h>100);
// a size set by hand stays
S.ayar.kartMin={odak:420};boyutla();await bek(30);
dogru('kartMin kept',q('[data-kart=odak]').getBoundingClientRect().height>=419);
// back to filling
delete S.ayar.kartMin;ayarAc();await bek(80);q('#aKartYuk [data-v=""]').click();await bek(80);
for(const x of document.querySelectorAll('dialog[open]'))x.close();
esit('default again',[S.ayar.kartYuk,K.dataset.kartYuk],[undefined,'']);
dogru('fills again',[...document.querySelectorAll('main [data-kart]')].some(e=>e.classList.contains('buyur')));
