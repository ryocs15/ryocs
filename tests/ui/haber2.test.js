// dil: tr
// News: save for later, copy link, mark all read, a clear message when filters hide everything, damaged saved entries
const {esit,dogru,q,bek}=RT;
const acilan=[];const f0=window.fetch;
window.fetch=async(p,a)=>{if(p==='/api/open'){acilan.push(JSON.parse(a.body).url);return{json:async()=>({durum:'ok'})}}return f0(p,a)};
for(const x of document.querySelectorAll('dialog[open]'))x.close();
const k=haberK();haberV[k]=[{b:'Birinci haber',l:'https://ornek.com/1',k:'Kaynak A',t:''},{b:'İkinci haber',l:'https://ornek.com/2',k:'Kaynak B',t:''}];S.haberKayit=[];S.ayar.haberGizli=[];S.ayar.haberKelime='';haberCiz();
const sec=t=>[...q('#menu').querySelectorAll('button')].find(b=>b.textContent===t);
q('#haberler [data-h="0"]').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:10,clientY:10}));await bek(30);
esit('story menu',[...q('#menu').querySelectorAll('button')].map(b=>b.textContent),['Sonra oku','Bağlantıyı kopyala','Kaynak A haberlerini gizle']);
sec('Sonra oku').click();esit('saved',S.haberKayit.map(x=>[x.b,x.l]),[['Birinci haber','https://ornek.com/1']]);
haberMenu(10,10,haberV[k][0]);await bek(20);dogru('saved: remove offered',!!sec('Kaydedilenlerden çıkar'));q('#menu').hidePopover();
hbKayitPencere();await bek(30);q('#sor [data-ka="0"]').click();esit('opened from the saved list',acilan,['https://ornek.com/1']);
q('#sor [data-ks="0"]').click();await bek(30);esit('removed from the saved list',S.haberKayit,[]);
for(const x of document.querySelectorAll('dialog[open]'))x.close();
// mark all read
hbOkundu=[];hbTumOkundu();esit('all read',[...q('#haberler').querySelectorAll('.hb')].map(b=>b.classList.contains('okundu')),[true,true]);
// filters hide everything: the message says so
S.ayar.haberKelime='haber';haberCiz();esit('all filtered: explained',q('#haberler').textContent,'2 haberin hepsi gizlenen kaynaklara ya da kelimelere takıldı.');
S.ayar.haberKelime='';
// damaged saved entries are dropped when the data is loaded
esit('damaged entries dropped',normal({haberKayit:[null,{l:1},{b:'x',l:'https://a'},'x']}).haberKayit,[{b:'x',l:'https://a'}]);
