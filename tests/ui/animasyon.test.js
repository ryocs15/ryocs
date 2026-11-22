// dil: tr
// Animation pass: new rows on a refresh slide in (only the new ones, not on the first drawing), switching tabs fades the content in,
// deleted rows shrink away, menus and the toast close with a transition; with animations off nothing of that runs
const {esit,dogru,q,bek}=RT;
for(const x of document.querySelectorAll('dialog[open]'))x.close();
K.dataset.anim='';
// news: first drawing still, then only the new story
kartAc('haber',true);const k=haberK();satirSon.delete('haber:'+k);
haberV[k]=[{b:'Bir',l:'https://x/1',k:'A',t:''},{b:'İki',l:'https://x/2',k:'A',t:''}];haberCiz();
esit('first drawing: no row animates',q('#haberler').querySelectorAll('.satirGir').length,0);
haberV[k]=[{b:'Yeni',l:'https://x/3',k:'A',t:''},...haberV[k]];haberCiz();
esit('only the new row animates',[...q('#haberler').querySelectorAll('.satirGir')].map(b=>b.querySelector('b').textContent),['Yeni']);
// switching the task tab fades the list in
sekme='gorev';ciz();q('#sekme [data-v="aliskanlik"]').click();
dogru('tab switch: content fades in',q('#todos').classList.contains('gecis'));
q('#sekme [data-v="gorev"]').click();
// hiding a source: its rows shrink away first, then the list is redrawn without them
S.ayar.haberGizli=[];haberCiz();hbKaynakGizle('A');
dogru('rows leave with an animation',q('#haberler').querySelectorAll('.hb.cik').length===3);
await bek(400);
esit('then hidden',[A('haberGizli'),[...q('#haberler').querySelectorAll('.hb')].filter(b=>b.title.endsWith('— A')).length],[['A'],0]);
S.ayar.haberGizli=[];
// menus and the toast close with a transition the browser understands
dogru('menu closes with a transition',getComputedStyle(q('#menu')).transitionProperty.includes('opacity')&&/allow-discrete/.test(getComputedStyle(q('#menu')).transitionBehavior));
dogru('toast closes with a transition',getComputedStyle(q('#tost')).transitionProperty.includes('opacity'));
// animations off: nothing is added
K.dataset.anim='off';
haberV[k]=[{b:'Başka',l:'https://x/4',k:'B',t:''},...haberV[k]];haberCiz();
esit('off: no slide-in',q('#haberler').querySelectorAll('.satirGir').length,0);
q('#todos').classList.remove('gecis');gecisOynat('#todos');
dogru('off: no fade',!q('#todos').classList.contains('gecis'));
K.dataset.anim='';
