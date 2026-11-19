// dil: en
// English currency card (ECB, per euro): values in dollars, USD in the list skipped, gold in ounces by default
const {esit,dogru}=RT;
kurV=ecbAyir('<e><Cube><Cube time="2026-11-19"><Cube currency="USD" rate="1.10"/><Cube currency="GBP" rate="0.88"/><Cube currency="JPY" rate="165"/></Cube></Cube></e>',4000,null);kurHata=false;
const satir=()=>[...document.querySelectorAll('#kur .kurS')].map(s=>s.textContent.replace(/\s+/g,' ').trim());
S.ayar.kurListe='USD, EUR, GBP, JPY';kurCiz();
esit('rows',satir().length,4);
dogru('EUR and GBP in dollars',satir()[0].includes('$1.1000')&&satir()[1].includes('$1.2500'));
dogru('JPY and gold',satir()[2].includes('$0.006667')&&satir()[3].includes('$4,000.00')&&satir()[3].includes('ounce'));
kurV=ecbAyir('<e><Cube><Cube time="2026-11-19"><Cube currency="USD" rate="1.10"/></Cube></Cube></e>',4000,null,{USD:1,PLN:4});S.ayar.kurListe='PLN';kurCiz();
dogru('PLN cross: 1 ÷ 4 dollars',satir()[0].includes('$0.2500')&&satir()[0].includes('cross'));
// any home currency: values are converted into it, gold too; TRY home switches to the TCMB source
kurV=ecbAyir('<e><Cube><Cube time="2026-11-19"><Cube currency="USD" rate="1.10"/><Cube currency="GBP" rate="0.88"/></Cube></Cube></e>',4000,null,{USD:1,PLN:4});
S.ayar.kurTaban='pln';S.ayar.kurListe='EUR, USD, PLN';kurCiz();
dogru('EUR in PLN = 4.4, USD in PLN = 4, PLN itself skipped',satir().length===3&&satir()[0].includes('EUR → PLN')&&satir()[0].includes('4.4000')&&satir()[1].includes('4.0000'));
dogru('gold in PLN (ounce)',satir()[2].includes('16,000.00'));
S.ayar.kurTaban='XXX';kurCiz();dogru('unknown home currency',RT.q('#kur').textContent.includes('Home currency not found: XXX'));
S.ayar.kurTaban='TRY';kurCiz();dogru('TRY home waits for TCMB rates',RT.q('#kur').textContent.includes('Loading'));
esit('defaults from the country (US)',[TABAN0,ONS0,kurVarsListe('USD'),kurVarsListe('EUR')],['USD','ons',['EUR','GBP'],['USD','GBP']]);
