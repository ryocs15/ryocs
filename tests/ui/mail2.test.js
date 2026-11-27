// dil: tr
// Gmail card: archive, mark as unread, a failed archive put back, preview text, new-mail notification
const {esit,dogru,q,bek}=RT;
const istek=[],bild=[];let islemCevap={durum:'ok'},cevap=null;const f0=window.fetch;
window.fetch=async(p,a)=>{
  if(p.startsWith('/api/mail?'))return{json:async()=>cevap};
  if(p==='/api/mail_islem'){istek.push(JSON.parse(a.body));return{json:async()=>islemCevap}}
  if(p==='/api/notify'){bild.push(JSON.parse(a.body));return{json:async()=>({durum:'ok'})}}
  return f0(p,a)};
for(const x of document.querySelectorAll('dialog[open]'))x.close();
S.ayar.rahatsiz=false;kartAc('mail',true);await bek(100);
const liste=()=>[{id:'a',konu:'Fatura',kimden:'"Banka" <b@x>',zincir:'a',yeni:true,ozet:'Don&#39;t forget &amp; pay'},{id:'b',konu:'Eski',kimden:'c@x',zincir:'b',yeni:false}];
cevap={durum:'ok',veri:{okunmamis:1,liste:liste()}};mailGorulen=null;await mail(true);
esit('first fetch: no notification',bild.length,0);
dogru('preview decoded in the tooltip',q('#mail [data-m="0"]').title.endsWith("Don't forget & pay"));
// archive: gone at once; unread count drops
const sec=t=>[...q('#menu').querySelectorAll('button')].find(b=>b.textContent===t);
mailMenu(10,10,mailV.liste[0]);sec('Arşivle').click();await bek(30);
esit('archived',[istek.at(-1),mailV.liste.map(m=>m.id),q('#mailSay').textContent],[{id:'a',islem:'arsiv',hesap:''},['b'],'']);
// a refresh that started before the archive still has it: it stays out, the count does not come back
cevap={durum:'ok',veri:{okunmamis:1,liste:liste()}};await mail(true);
esit('late refresh: archived mail stays out',[mailV.liste.map(m=>m.id),q('#mailSay').textContent],[['b'],'']);
cevap={durum:'ok',veri:{okunmamis:0,liste:[liste()[1]]}};await mail(true);esit('forgotten once gone',mailArsiv.size,0);
// mark as unread
mailMenu(10,10,mailV.liste[0]);sec('Okunmadı olarak işaretle').click();await bek(30);
esit('unread',[istek.at(-1),mailV.liste[0].yeni,q('#mailSay').textContent],[{id:'b',islem:'okunmadi',hesap:''},true,'1']);
// a refused archive puts the mail back in its place
islemCevap={durum:'403: x'};mailMenu(10,10,mailV.liste[0]);sec('Arşivle').click();await bek(50);
esit('refused archive: back',[mailV.liste.map(m=>m.id),q('#mailSay').textContent,q('#tost').textContent.startsWith('Yapılamadı')],[['b'],'1',true]);
islemCevap={durum:'ok'};
// new unread mail since the last fetch: one notification; off in the card settings: none
cevap={durum:'ok',veri:{okunmamis:2,liste:[{id:'n',konu:'Toplantı',kimden:'"Ayşe" <a@x>',zincir:'n',yeni:true},...liste()]}};await mail(true);
esit('new mail notified',[bild.length,bild[0]?.baslik,bild[0]?.metin],[1,'Yeni mail','Ayşe: Toplantı']);
await mail(true);esit('the same mail again: no second notification',bild.length,1);
S.ayar.mailBildir=false;cevap={durum:'ok',veri:{okunmamis:3,liste:[{id:'z',konu:'X',kimden:'x@y',zincir:'z',yeni:true},...cevap.veri.liste]}};await mail(true);
esit('notifications off',bild.length,1);
