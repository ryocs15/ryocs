// dil: pl
// Polish: texts come from src/dil/pl.json, numbers and dates from the pl locale, news from Polsat News, wind directions in Polish
const {esit,dogru,q}=RT;
esit('language',[DIL,document.documentElement.lang],['pl','pl']);
esit('static text from the table',q('#ayB').textContent,'Ustawienia');
esit('L()',[KART_AD.hava,KART_AD.gorevler,L('Kaydet','Save')],['Pogoda','Zadania','Zapisz']);
esit('Lf() fills the placeholder',Lf('Dünden {0}° daha sıcak','{0}° warmer than yesterday',3),'o 3° cieplej niż wczoraj');
esit('news tabs',Object.keys(HKAT)[0]+' '+hKaynak('pl_top'),'pl_top Polsat News');
dogru('language picker has Polish and Spanish',['pl','es'].every(v=>q(`#aDil option[value=${v}]`)));
dogru('no English left in the settings labels',![...document.querySelectorAll('#ayarlar .ayNav button')].some(b=>/^(Profile|Appearance|Notifications|Privacy and security)$/.test(b.textContent.trim())));
