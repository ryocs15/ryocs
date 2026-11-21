// dil: tr
// Currency card 1.1.7: no alert from old cached rates, gold/silver alerts with their unit, gold daily change, converter
const {esit,dogru,q,bek}=RT;
const bild=[];const f0=window.fetch;
window.fetch=async(p,a)=>{if(p==='/api/notify'){bild.push(JSON.parse(a.body));return{json:async()=>({durum:'ok'})}}return f0(p,a)};
for(const x of document.querySelectorAll('dialog[open]'))x.close();
S.ayar.rahatsiz=false;S.ayar.kurTaban='TRY';S.ayar.kurListe='USD, EUR';S.ayar.altinBirim='gram';S.ayar.kurDegisim=true;
const xml=`<Tarih_Date Tarih="20.11.2026"><Currency CurrencyCode="USD"><Unit>1</Unit><ForexBuying>40</ForexBuying><ForexSelling>40</ForexSelling><BanknoteSelling>40</BanknoteSelling></Currency>`
  +`<Currency CurrencyCode="EUR"><Unit>1</Unit><ForexBuying>46</ForexBuying><ForexSelling>46</ForexSelling><BanknoteSelling>46</BanknoteSelling></Currency></Tarih_Date>`;
const taze=()=>{const v=kurAyir(xml,3110.35,40);v.mDun={ons:3000,ag:40,usd:40};return v};
// an alert is not fired (nor removed) by rates cached long ago
S.ayar.kurAlarm={USD:{y:'>',v:39,b:'TRY'}};kurV=taze();kurV.t=Date.now()-3*864e5;kurHata=false;bild.length=0;kurCiz();
esit('old cache: no alert, kept',[bild.length,!!S.ayar.kurAlarm.USD],[0,true]);
kurV.t=Date.now();kurCiz();
esit('fresh rates: alert fires and is removed',[bild[0]?.metin,!!S.ayar.kurAlarm.USD],['USD → TRY: ₺40,00 (> ₺39,00)',false]);
// gold: daily change in TRY, clickable for an alert; the alert keeps its unit
kurV=taze();kurCiz();
const altin=[...document.querySelectorAll('#kur .kurS')].find(s=>s.textContent.startsWith('Altın'));
dogru('gold change: ▲ 3,68%',altin.textContent.includes('▲ 3,68%'));
esit('gold row opens the alert menu',altin.dataset.c,'XAU');
window.sor=async()=>['4100'];await kurAlarm('XAU');
esit('gold alert with unit',S.ayar.kurAlarm.XAU,{y:'>',v:4100,b:'TRY',u:'gram'});
S.ayar.altinBirim='ons';esit('per ounce: the gram alert does not count',Object.keys(kurAlarmlar()),[]);
S.ayar.altinBirim='gram';kurV.ons=3200;bild.length=0;kurCiz();
esit('gold alert fires',[bild[0]?.baslik,!!S.ayar.kurAlarm.XAU],['Fiyat alarmı',false]);
// converter: 100 USD = 4.000 TL; reversed: 1.000 TL = 25 USD; typing keeps the input
S.ayar.kurCevir=false;kurV=taze();kurCiz();await bek(20);
dogru('closed by default',q('#kurCev').hidden);q('#kurCevB').click();
esit('title button opens it, remembered',[q('#kurCev').hidden,S.ayar.kurCevir,q('#kurCevB').getAttribute('aria-pressed')],[false,true,'true']);
const c=q('#kurCev');esit('converter built',c.querySelector('b')?.textContent,'= 4.000');
c.querySelector('select').value='EUR';c.querySelector('select').dispatchEvent(new Event('change',{bubbles:true}));
esit('EUR',c.querySelector('b').textContent,'= 4.600');
const inp=c.querySelector('input');inp.value='1000';inp.dispatchEvent(new Event('input',{bubbles:true}));c.querySelector('select').value='USD';c.querySelector('select').dispatchEvent(new Event('change',{bubbles:true}));
c.querySelector('[data-yon]').click();
esit('reversed',[c.querySelector('b').textContent,c.querySelector('input').value,c.querySelector('select').value],['= 25','1000','USD']);
kurCiz();esit('redraw keeps the same input',c.querySelector('input')===q('#kurCev input'),true);
