// dil: tr
// Rate alert (one-shot notification when the target is crossed) and clipboard row → task / note
const {esit,dogru,q}=RT;
const xml=v=>`<Tarih_Date Tarih="19.11.2026"><Currency CurrencyCode="USD"><Unit>1</Unit><ForexBuying>${v}</ForexBuying><ForexSelling>${v}</ForexSelling><BanknoteSelling>${v}</BanknoteSelling></Currency>`
  +`<Currency CurrencyCode="EUR"><Unit>1</Unit><ForexBuying>46</ForexBuying><ForexSelling>46</ForexSelling><BanknoteSelling>46</BanknoteSelling></Currency></Tarih_Date>`;
const giden=[],f0=window.fetch;window.fetch=async(p,a)=>{if(p==='/api/notify'){giden.push(a.body);return{json:async()=>({durum:'ok'})}}return f0(p,a)};O.mod='';
S.ayar.kurListe='USD, EUR';S.ayar.kurTur='';delete S.ayar.kurAlarm;
kurV=kurAyir(xml('40.00'),3000,null);kurHata=false;kurCiz();
dogru('rows carry their code',!!q('#kur [data-c="USD"]')&&!!q('#kur [data-c="EUR"]'));
S.ayar.kurAlarm={USD:{y:'>',v:41,b:'TRY'}};kurCiz();
dogru('bell on the row with an alert',!!q('#kur [data-c="USD"] .kurA')&&!q('#kur [data-c="EUR"] .kurA'));
esit('not crossed: no notification',giden.length,0);
kurV=kurAyir(xml('41.20'),3000,null);kurCiz();
esit('crossed: one notification',giden.length,1);
dogru('alert removed after firing',!A('kurAlarm').USD&&!q('#kur .kurA'));
kurCiz();esit('fires only once',giden.length,1);
S.ayar.kurAlarm={EUR:{y:'<',v:45,b:'TRY'}};kurCiz();esit('below target not reached',giden.length,1);
esit('current rate for the dialog',kurDeg('USD'),41.2);
// an alert set in another home currency does not fire and is kept
S.ayar.kurAlarm={USD:{y:'<',v:50,b:'EUR'}};kurCiz();
dogru('other home currency: no bell, no notification, kept',!q('#kur .kurA')&&giden.length===1&&!!A('kurAlarm').USD);
// alerts from a damaged or edited copy are ignored, nothing reaches the HTML unescaped
S.ayar.kurAlarm={USD:{y:'"><img src=x onerror=alert(1)>',v:1},EUR:{y:'>',v:'<b>x</b>'}};kurCiz();
dogru('bad alerts ignored',!q('#kur .kurA')&&!q('#kur img')&&!q('#kur b b'));
// replacing all data: no autosave during the request; on failure the data stays
{const f1=window.fetch,eski=S;let sirada;window.fetch=async(p,a)=>{if(p==='/api/backups/restore'){sirada=S;return{json:async()=>({durum:'hata'})}}return f1(p,a)};
 await veriDegistir('/api/backups/restore',{ad:'x'});window.fetch=f1;
 dogru('S empty during the request',sirada===null);dogru('S back after a failure',S===eski);}
// a rate that is not available: no dialog, so no alert with a guessed direction
await kurAlarm('XYZ');dogru('unknown rate: no alert dialog',!q('#sor').textContent.includes('XYZ'));
// clipboard row → task / note
panoCiz({durum:'ok',veri:{liste:['Faturayı öde\nikinci satır','x']}});
const n0=S.todos.length,b=q('#pano [data-p="0"]');
b.dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:10,clientY:10}));
const m=[...document.querySelectorAll('#menu button')];
esit('menu items',m.map(x=>x.textContent),['Görev olarak ekle','Nota ekle']);
m[0].click();
esit('task added from the first line',S.todos.slice(n0).map(t=>t.t),['Faturayı öde']);
const not=S.notlar[S.aktifNot],once=not.metin;
b.dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:10,clientY:10}));
document.querySelectorAll('#menu button')[1].click();
dogru('appended to the current note',not.metin.startsWith(once)&&not.metin.endsWith('Faturayı öde\nikinci satır'));
window.fetch=f0;
// damaged entries in a loaded or imported file: the panel still draws
{const eski=S;S=normal({todos:[null,{t:'ok'},{d:true}],notlar:[{ad:'x'},null,{metin:'m'}]});
 let h='';try{ciz();notCiz()}catch(e){h=String(e)}
 esit('damaged entries do not crash',h,'');esit('tasks without text dropped',S.todos.map(t=>t.t),['ok']);
 esit('notes get text and a name',S.notlar.map(n=>[n.ad,n.metin]),[['x',''],['Not','m']]);
 S=eski;ciz();notCiz();}
// one-click update: saves first, then asks Rust to install; never opens a web page; a failure is shown and can be retried
{const f1=window.fetch,yol=[];let cevap={durum:'ok',veri:'9.9.9'};
 window.fetch=async(p,a)=>{yol.push(p);if(p==='/api/guncelle'||p==='/api/state'&&a?.method==='POST')return{json:async()=>(p==='/api/guncelle'?cevap:{durum:'ok'})};return f1(p,a)};
 cevap={durum:'HTTP 404'};await gunKur();
 dogru('failure shown',q('#aGunD').textContent.includes('HTTP 404'));
 cevap={durum:'ok',veri:'9.9.9'};await gunKur();window.fetch=f1;
 const i=yol.lastIndexOf('/api/guncelle');
 dogru('saved before installing',yol.slice(0,i).includes('/api/state'));
 dogru('no web page opened',!yol.includes('/api/open'));
 dogru('second click while installing does nothing',(await gunKur(),yol.filter(p=>p==='/api/guncelle').length===2));}
