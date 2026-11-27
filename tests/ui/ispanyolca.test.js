// dil: es
// Spanish: texts from src/dil/es.json, news from El País
const {esit,dogru,q}=RT;
esit('language',[DIL,document.documentElement.lang],['es','es']);
esit('static text from the table',q('#ayB').textContent,'Ajustes');
esit('L()',[KART_AD.hava,KART_AD.gorevler,L('Kaydet','Save')],['Tiempo','Tareas','Guardar']);
esit('Lf() fills the placeholder',Lf('Dünden {0}° daha sıcak','{0}° warmer than yesterday',3),'3° más cálido que ayer');
esit('news tabs',Object.keys(HKAT)[0]+' '+hKaynak('es_top'),'es_top El País');
dogru('language picker has Polish and Spanish',['pl','es'].every(v=>q(`#aDil option[value=${v}]`)));
dogru('no English left in the settings labels',![...document.querySelectorAll('#ayarlar .ayNav button')].some(b=>/^(Profile|Appearance|Notifications|Privacy and security)$/.test(b.textContent.trim())));
esit('no plural forms: the plain translation',Ln(' görev',' tasks',5),' tareas');
