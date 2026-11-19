// dil: tr
// Currency card settings: chosen codes, buying/selling, change arrow, gold unit, silver, missing codes
const {esit,dogru,q}=RT;
const xml=g=>`<Tarih_Date Tarih="${g}"><Currency CurrencyCode="USD"><Unit>1</Unit><ForexBuying>40.00</ForexBuying><ForexSelling>40.10</ForexSelling><BanknoteSelling>40.20</BanknoteSelling></Currency>`
  +`<Currency CurrencyCode="EUR"><Unit>1</Unit><ForexBuying>46</ForexBuying><ForexSelling>${g==='18.11.2026'?'45.00':'46.00'}</ForexSelling><BanknoteSelling></BanknoteSelling></Currency>`
  +`<Currency CurrencyCode="JPY"><Unit>100</Unit><ForexBuying>25</ForexBuying><ForexSelling>26</ForexSelling><BanknoteSelling>27</BanknoteSelling></Currency></Tarih_Date>`;
const dun=kurAyir(xml('18.11.2026'),3000,null);kurV=kurAyir(xml('19.11.2026'),3110.35,40);kurV.dun=dun.k;kurHata=false;
const satir=()=>[...document.querySelectorAll('#kur .kurS')].map(s=>s.textContent.replace(/\s+/g,' ').trim());
esit('JPY per unit',kurV.k.JPY.s,.26);
kurCiz();
esit('default: dollar, euro, gold',satir().length,3);
dogru('euro rose 2.22 %',satir()[1].includes('▲ 2,22%'));
dogru('gram gold = ounce × dollar ÷ 31.1035',satir()[2].includes('₺4.010,00'));
S.ayar.kurListe='jpy, eur xyz,JPY';S.ayar.kurTur='al';kurCiz();
esit('typed codes in order, duplicates removed',satir().map(s=>s.match(/JPY|EUR|Altın/)?.[0]),['JPY','EUR','Altın']);
dogru('buying JPY',satir()[0].includes('₺0,2500'));
dogru('missing code named, buying in source line',q('#kurAlt').textContent.includes('bulunamadı: XYZ')&&q('#kurAlt').textContent.includes('alış kuru'));
S.ayar.kurListe='EUR';S.ayar.kurTur='ef';kurCiz();
dogru('empty banknote rate shows a dash',satir()[0].includes('—'));
S.ayar.kurDegisim=false;S.ayar.altinBirim='ons';S.ayar.gumus=true;S.ayar.kurTur='';kurCiz();
dogru('no change arrow',!q('#kur .kurD'));
dogru('ounce gold and silver',satir()[1].includes('₺124.725,04')&&satir()[2].startsWith('Gümüş')&&satir()[2].includes('₺1.604,00'));
// the card settings window lists the new settings
kartAyar('kur');
dogru('settings window',['kurListe','kurTur','altinBirim','gumus','kurDegisim'].every(k=>q(`.ic [data-a="${k}"]`))&&!!q('.ic [data-g="altin"]'));
// codes TCMB lacks come from world rates per dollar; the feed's own value wins; bad entries are ignored
kurV=kurAyir(xml('19.11.2026'),3110.35,40,{USD:1,PLN:4,EUR:2,TRY:50,pln:9,XYZ:-1,'<b>':3});kurHata=false;S.ayar.kurListe='PLN, EUR, XYZ';kurCiz();
esit('cross codes',kurV.w,['PLN']);
dogru('PLN = 40.10 ÷ 4, marked cross',satir()[0].includes('₺10,03')&&satir()[0].includes('çapraz'));
dogru('EUR from TCMB, attribution shown',satir()[1].includes('₺46,00')&&q('#kurAlt').textContent.includes('ExchangeRate-API')&&q('#kurAlt').textContent.includes('bulunamadı: XYZ'));
