// dil: tr
// Breaking news from several outlets: one list newest first, the same story merged with "+n kaynak", a feed's wrong time zone fixed,
// hidden outlets, and the calm "new" dot only on stories that came since you last looked
const {esit,dogru,q,bek}=RT;
const f0=window.fetch,dk=m=>new Date(Date.now()-m*6e4);
const rss=l=>`<?xml version="1.0"?><rss><channel>${l.map(([b,u,t])=>`<item><title>${b}</title><link>${u}</link><pubDate>${t}</pubDate></item>`).join('')}</channel></rss>`;
// CNN Türk writes Turkish time and labels it GMT: 20 minutes ago in Turkey reads as 3 h ahead
const cnnSaat=new Date(Date.now()-20*6e4+108e5).toUTCString();
let akis=[
  {k:'TRT Haber',x:rss([["Ankara'da 4.5 büyüklüğünde deprem meydana geldi",'https://trt/1',dk(10).toUTCString()]])},
  {k:'Sabah',x:rss([["Son dakika: Ankara'da 4.5 büyüklüğünde deprem",'https://sabah/1',dk(5).toUTCString()]])},
  {k:'CNN Türk',x:rss([['Merkez Bankası faiz kararını açıkladı','https://cnn/1',cnnSaat]])},
  {k:'Sözcü',x:rss([['Galatasaray yeni transferini duyurdu','https://sozcu/1',dk(2).toUTCString().replace('GMT','&#x2B;0000')]])},
];
let cevap=()=>({durum:'ok',veri:akis});
window.fetch=async(p,a)=>p==='/api/haber'&&JSON.parse(a.body).kat==='sondakika'?{json:async()=>cevap()}:f0(p,a);
for(const x of document.querySelectorAll('dialog[open]'))x.close();
hbGor={};try{localStorage.removeItem('hbGorulen')}catch{}
S.ayar.haberKat='sondakika';S.ayar.haberGizli=[];S.ayar.haberKelime='';kartAc('haber',true);await bek(50);
await haber(true);await bek(30);
const satir=()=>[...q('#haberler').querySelectorAll('.hb')].map(b=>b.querySelector('b').textContent+' | '+b.querySelector('small').textContent.split(' · ')[0]);
const nokta=()=>[...q('#haberler').querySelectorAll('.hb')].filter(b=>b.querySelector('.hbNokta')).map(b=>b.querySelector('b').textContent);
esit('merged, newest first',satir(),['Galatasaray yeni transferini duyurdu | Sözcü',"Ankara'da 4.5 büyüklüğünde deprem meydana geldi | TRT Haber +1 kaynak",'Merkez Bankası faiz kararını açıkladı | CNN Türk']);
dogru('wrong time zone fixed: 20 min ago, not in the future',Math.abs(Date.parse(haberV.sondakika[2].t)-(Date.now()-20*6e4))<6e4);
dogru('source count under the card',q('#hbAlt').textContent.startsWith('4 kaynak'));
esit('first load: no dots',nokta(),[]);

// a new story arrives: only it gets the dot; the next refresh takes it away
akis=[...akis,{k:'Milliyet',x:rss([['İstanbul Boğazı gemi trafiğine kapatıldı','https://milliyet/1',dk(1).toUTCString()]])}];
await haber(true);await bek(30);
esit('only the new story has a dot',nokta(),['İstanbul Boğazı gemi trafiğine kapatıldı']);
await haber(true);await bek(30);
esit('seen: dot gone on the next refresh',nokta(),[]);

// while the window is hidden new stories keep their dot until you have looked
gorunmez=true;
akis=[...akis,{k:'Habertürk',x:rss([['Meteoroloji fırtına uyarısı yaptı','https://haberturk/1',dk(0.5).toUTCString()]])}];
await haber(true);await bek(30);
gorunmez=false;hbIsle(haberK());haberCiz();
esit('came while hidden: still new',nokta(),['Meteoroloji fırtına uyarısı yaptı']);

// a hidden outlet: the merged story shows the next outlet; the menu offers the other outlets
const deprem=()=>[...q('#haberler').querySelectorAll('.hb')].find(b=>b.textContent.includes('deprem'));
deprem().dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:10,clientY:10}));await bek(30);
dogru('open at the other outlet',[...q('#menu').querySelectorAll('button')].some(b=>b.textContent==='Kaynakta aç: Sabah'));
q('#menu').hidePopover();
S.ayar.haberGizli=['TRT Haber'];haberCiz();
esit('hidden outlet: next one shows',deprem().querySelector('small').textContent.split(' · ')[0],'Sabah');
S.ayar.haberGizli=[];

// different stories are not merged; all feeds failing shows the error, the last list stays
esit('common words do not merge stories',hbBirlestir([{b:'Bakan için yeni karar sonra açıklandı',l:'a',k:'A',t:dk(1).toISOString()},{b:'Takım için yeni teknik direktör sonra geliyor',l:'b',k:'B',t:dk(2).toISOString()}]).length,2);
esit('different stories stay apart',hbBirlestir([{b:'Ankara deprem sarsıldı',l:'a',k:'A',t:dk(1).toISOString()},{b:'Ankara trafik kazası oldu',l:'b',k:'B',t:dk(2).toISOString()}]).length,2);
cevap=()=>({durum:'No internet connection'});
await haber(true);await bek(30);
dogru('error: the last list stays',q('#haberler').querySelectorAll('.hb').length>=4&&q('#hbAlt').textContent.includes('yenilenemedi'));
window.fetch=f0;S.ayar.haberKat='gundem';
