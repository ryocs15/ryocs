// dil: tr
// Several Gmail accounts: chips with unread counts, "all" merged with a colour dot per account, one account only, actions carry the
// account, opening uses that account's Gmail, a disconnected account offers to reconnect, the notification says the account,
// the settings list extra accounts and remove them
const {esit,dogru,q,bek}=RT;
const islem=[],bildirim=[],silinen=[];let acilan=null;const f0=window.fetch;
const m=(id,h,konu,dk,yeni)=>({id,hesap:h,konu,kimden:'X <x@y>',zincir:'z'+id,t:Date.now()-dk*6e4,yeni});
let cevap={durum:'ok',veri:{okunmamis:3,liste:[m('b','is@gmail.com','İş 1',3,true),m('a','ben@gmail.com','Ana 1',5,true),m('c','is@gmail.com','İş 2',60,false),m('d','ben@gmail.com','Ana 2',90,true)],
  hesaplar:[{eposta:'ben@gmail.com',ana:true,okunmamis:2},{eposta:'is@gmail.com',okunmamis:1},{eposta:'eski@gmail.com',hata:'bagli_degil'}]}};
let ayar={durum:'ok',veri:{google:'ok',spotify:'kimlik_yok',posta:[{eposta:'is@gmail.com',durum:'ok'},{eposta:'eski@gmail.com',durum:'bagli_degil'}],kisayollar:[],surum:'x',sehir:'A'}};
window.fetch=async(p,a)=>{
  const j=v=>({json:async()=>v});
  if(p.startsWith('/api/mail?'))return j(cevap);
  if(p==='/api/mail_islem'){islem.push(JSON.parse(a.body));return j({durum:'ok'})}
  if(p==='/api/open'){acilan=JSON.parse(a.body).url;return j({durum:'ok'})}
  if(p==='/api/notify'){bildirim.push(JSON.parse(a.body));return j({durum:'ok'})}
  if(p==='/api/config')return j(ayar);
  if(p==='/api/posta_sil'){silinen.push(JSON.parse(a.body).eposta);return j({durum:'ok'})}
  return f0(p,a)};
for(const x of document.querySelectorAll('dialog[open]'))x.close();
delete S.ayar.mailHesap;kartAc('mail',true);await bek(100);await mail(true);
const cipler=()=>[...q('#mail').querySelectorAll('[data-mh]')].map(b=>b.textContent);
const satir=()=>[...q('#mail').querySelectorAll('.hb b')].map(b=>b.textContent);
esit('chips with counts',cipler(),['Tümü · 3','ben · 2','is · 1','eski !']);
esit('all: merged, newest first',satir(),['İş 1','Ana 1','İş 2','Ana 2']);
dogru('all: a colour dot per account',q('#mail .hb .mhN')!==null);
esit('badge: total',q('#mailSay').textContent,'3');
// one account
q('#mail [data-mh="is@gmail.com"]').click();
esit('one account',[satir(),q('#mailSay').textContent,S.ayar.mailHesap],[['İş 1','İş 2'],'1','is@gmail.com']);
// opening uses that account; marking read carries the account and lowers its count
q('#mail .hb').click();await bek(30);
esit('opens in that account',acilan,'https://mail.google.com/mail/u/is%40gmail.com/#inbox/zb');
esit('action carries the account',islem.at(-1),{id:'b',islem:'okundu',hesap:'is@gmail.com'});
esit('its count went down',[q('#mailSay').textContent,cipler()[2]],['','is']);
// a disconnected account: reconnect offered
q('#mail [data-mh="eski@gmail.com"]').click();
dogru('reconnect offered',q('#mail').textContent.includes('bağlantısı koptu')&&!!q('#mail [data-mbagla]'));
q('#mail [data-mh=""]').click();esit('back to all',S.ayar.mailHesap,undefined);
// new mail notification names the account
cevap.veri.liste=[m('e','is@gmail.com','Yeni teklif',0,true),...cevap.veri.liste];await mail(true);await bek(30);
esit('notification says the account',bildirim.at(-1)?.baslik,'Yeni mail · is@gmail.com');
// a newly added account: its unread inbox is remembered, not announced
const n0=bildirim.length;
cevap.veri.liste=[...cevap.veri.liste,m('y1','yeni@gmail.com','Eski okunmamış',200,true),m('y2','yeni@gmail.com','Bir tane daha',300,true)];
cevap.veri.hesaplar=[...cevap.veri.hesaplar,{eposta:'yeni@gmail.com',okunmamis:2}];await mail(true);await bek(30);
esit('added account: no burst of notifications',bildirim.length,n0);
// settings: the extra accounts are listed and can be removed
ayarAc();await bek(150);
esit('settings list',[...q('#aPosta').querySelectorAll('.ar>span')].map(s=>s.childNodes[1].textContent),['is@gmail.com','eski@gmail.com']);
dogru('reconnect button for the broken one',q('#aPosta [data-pb]')!==null);
q('#aPosta [data-ps="is@gmail.com"]').click();await bek(80);
q('#sor form')?.requestSubmit();await bek(80);
esit('removed',silinen,['is@gmail.com']);
for(const x of document.querySelectorAll('dialog[open]'))x.close();
// Outlook: opens its own web link, the menu says Outlook, the action carries the account
cevap={durum:'ok',veri:{okunmamis:1,liste:[{id:'AAMk-x_y=',hesap:'ben@outlook.com',konu:'Outlook mail',kimden:'"Ayşe" <a@b>',t:Date.now(),yeni:true,ozet:'',link:'https://outlook.live.com/owa/?ItemID=AAMk'},
  {id:'AAMk2',hesap:'ben@outlook.com',konu:'Kötü link',kimden:'x',t:Date.now()-6e4,yeni:false,link:'https://kotu.example/x'}],
  hesaplar:[{eposta:'ben@gmail.com',ana:true,okunmamis:0},{eposta:'ben@outlook.com',okunmamis:1}]}};await mail(true);
q('#mail [data-mh="ben@outlook.com"]').click();
q('#mail .hb').click();await bek(30);
esit('Outlook: its web link',acilan,'https://outlook.live.com/owa/?ItemID=AAMk');
esit('Outlook action carries the account',islem.at(-1),{id:'AAMk-x_y=',islem:'okundu',hesap:'ben@outlook.com'});
q('#mail .hb:nth-of-type(2)').click();await bek(30);
esit('a strange link is not opened, Outlook on the web is',acilan,'https://outlook.live.com/mail/');
mailMenu(10,10,mailV.liste[0]);await bek(20);
dogru('menu says Outlook',[...q('#menu').querySelectorAll('button')].some(b=>b.textContent==='Outlook’ta aç'));q('#menu').hidePopover();
// colours: right-click a chip → change colour; a bad value never reaches the page; back to the default
q('#mail [data-mh="ben@outlook.com"]').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:10,clientY:10}));await bek(30);
[...q('#menu').querySelectorAll('button')].find(b=>b.textContent==='Rengini değiştir…').click();await bek(60);
q('#sor input[name=a0]').value='#123abc';q('#sor form').requestSubmit();await bek(60);
esit('colour saved',S.ayar.mailRenk,{'ben@outlook.com':'#123abc'});
dogru('chip uses it',q('#mail [data-mh="ben@outlook.com"] .mhN').style.getPropertyValue('--c')==='#123abc');
S.ayar.mailRenk={'ben@outlook.com':'red;background:url(x)'};mailCiz();
dogru('bad value ignored',!q('#mail').innerHTML.includes('url(x)'));
mailRenkSifirla('ben@outlook.com');esit('default again',S.ayar.mailRenk,undefined);
delete S.ayar.mailHesap;
// Outlook setup: the guide opens with the redirect address to copy, a wrong ID is refused
ayar.veri.microsoft='kimlik_yok';await postaEkle('microsoft');await bek(80);
dogru('guide with the redirect address',q('#sor').textContent.includes('http://localhost:8765/callback/microsoft'));
q('#sor input[name=id]').value='yanlis';q('#sor form').requestSubmit();await bek(30);
dogru('wrong ID refused',q('#sor .hata').textContent.startsWith('Uygulama (istemci) kimliği'));
for(const x of document.querySelectorAll('dialog[open]'))x.close();
// waiting for a sign-in: an account that was already connected does not count, one added after the dialog opened does
const bas=Math.floor(Date.now()/1000);
ayar.veri.posta=[{eposta:'eski@outlook.com',tur:'microsoft',durum:'ok',eklendi:bas-500}];
postaBekle('microsoft',bas);await bek(1700);
dogru('an old account is not taken as the new one',!q('#sor').textContent.includes('eklendi'));
ayar.veri.posta.push({eposta:'yeni@outlook.com',tur:'microsoft',durum:'ok',eklendi:bas+1});await bek(1700);
dogru('the new one is',q('#sor').textContent.includes('yeni@outlook.com eklendi'));
for(const x of document.querySelectorAll('dialog[open]'))x.close();
dogru('card is called Mail',q('[data-kart=mail] h2').textContent.startsWith('Mail'));
// one account only (older answer without "hesaplar"): no chips, as before
cevap={durum:'ok',veri:{okunmamis:1,liste:[{id:'z',konu:'Tek',kimden:'a@b',zincir:'zz',t:null,yeni:true}]}};await mail(true);
esit('single account: no chips',[cipler(),satir()],[[],['Tek']]);
window.fetch=f0;
