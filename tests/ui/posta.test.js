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
  if(p==='/api/mail')return j(cevap);
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
// settings: the extra accounts are listed and can be removed
ayarAc();await bek(150);
esit('settings list',[...q('#aPosta').querySelectorAll('.ar>span')].map(s=>s.firstChild.textContent),['is@gmail.com','eski@gmail.com']);
dogru('reconnect button for the broken one',q('#aPosta [data-pb]')!==null);
q('#aPosta [data-ps="is@gmail.com"]').click();await bek(80);
q('#sor form')?.requestSubmit();await bek(80);
esit('removed',silinen,['is@gmail.com']);
for(const x of document.querySelectorAll('dialog[open]'))x.close();
// one account only (older answer without "hesaplar"): no chips, as before
cevap={durum:'ok',veri:{okunmamis:1,liste:[{id:'z',konu:'Tek',kimden:'a@b',zincir:'zz',t:null,yeni:true}]}};await mail(true);
esit('single account: no chips',[cipler(),satir()],[[],['Tek']]);
window.fetch=f0;
