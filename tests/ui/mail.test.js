// dil: en
// Gmail card states and layout backups
const {esit,dogru,q,bek}=RT;
let cevap={durum:'izin_yok'},acilan=null,durum=null,okundu=[],okCevap={durum:'ok'};const f0=window.fetch;
window.fetch=async(p,a)=>{
  if(p.startsWith('/api/mail?'))return{json:async()=>cevap};
  if(p==='/api/mail_islem'){okundu.push(JSON.parse(a.body).id);return{json:async()=>okCevap}}
  if(p==='/api/open'){acilan=JSON.parse(a.body).url;return{json:async()=>({durum:'ok'})}}
  if(p==='/api/state'&&a?.method==='POST'){durum=JSON.parse(a.body);return new Promise(()=>{})} // no reload after applying a layout
  return f0(p,a)};

// a card added after a save existed starts closed
esit('new card starts closed',[kartAcik('mail'),S.ayar.kartBilinen?.includes('mail')],[false,true]);
kartAc('mail',true);await bek(100);
dogru('missing permission',q('#mail').textContent.startsWith('Update the Google permission for Gmail'));
cevap={durum:'403: { "error": { "message": "Gmail API has not been used in project 1 before or it is disabled." } }'};await mail(true);
dogru('API turned off',q('#mail').textContent.startsWith('The Gmail API is turned off'));
cevap={durum:'bagli_degil'};await mail(true);
dogru('not connected',q('#mail').textContent.startsWith('Google connection is missing or has expired.'));

cevap={durum:'ok',veri:{okunmamis:42,liste:[{id:'a1',konu:'Toplantı <b>notu</b>',kimden:'"Ali Veli" <ali@ornek.com>',zincir:'18c2ab',t:Date.now()-36e5,yeni:true},{id:'b2',konu:'',kimden:'yalniz@ornek.com',zincir:'kot/../x',t:null,yeni:true},{id:'c3',konu:'Eski',kimden:'x@y',zincir:'c3',t:null,yeni:false}]}};
await mail(true);
esit('list',[...q('#mail').querySelectorAll('.hb')].map(b=>b.textContent),['Toplantı <b>notu</b>Ali Veli · 1 h ago','(no subject)yalniz@ornek.com','Eskix@y']);
esit('unread badge',q('#mailSay').textContent,'42');
esit('read mail is not bold',[...q('#mail').querySelectorAll('.hb')].map(b=>b.classList.contains('yeni')),[true,true,false]);
dogru('subject is escaped',!q('#mail').innerHTML.includes('<b>notu'));
q('#mail .hb').click();esit('opens the thread',acilan,'https://mail.google.com/mail/u/0/#inbox/18c2ab');
await bek(50);esit('click marks it read',[okundu,q('#mail .hb').classList.contains('yeni'),q('#mailSay').textContent],[['a1'],false,'41']);
acilan=null;q('#mail .hb:nth-child(2)').click();esit('bad thread id is not opened',[acilan,okundu.length],[null,1]);
okCevap={durum:'izin_yok'};mailMenu(10,10,mailV.liste[1]);[...q('#menu').querySelectorAll('button')].find(b=>b.textContent==='Mark as read').click();await bek(50);
esit('menu: no permission puts it back',[okundu.at(-1),mailV.liste[1].yeni,q('#mailSay').textContent,q('#tost').textContent.startsWith('Update the Google permission')],['b2',true,'41',true]);
okCevap={durum:'ok'};mailMenu(10,10,mailV.liste[1]);[...q('#menu').querySelectorAll('button')].find(b=>b.textContent==='Mark as read').click();await bek(50);
esit('menu: marked read',[mailV.liste[1].yeni,q('#mailSay').textContent],[false,'40']);
cevap={durum:'ok',veri:{okunmamis:42,liste:[{id:'a1',konu:'x',kimden:'a@b',zincir:'18c2ab',t:null,yeni:true},{id:'b2',konu:'y',kimden:'a@b',zincir:'b2',t:null,yeni:true},{id:'c3',konu:'Eski',kimden:'x@y',zincir:'c3',t:null,yeni:false}]}};await mail(true);
esit('older refresh: marked mails stay read',[mailV.liste.map(m=>m.yeni),q('#mailSay').textContent],[[false,false,false],'40']);
cevap={durum:'ok',veri:{okunmamis:40,liste:[{id:'a1',konu:'x',kimden:'a@b',zincir:'18c2ab',t:null,yeni:false},{id:'b2',konu:'y',kimden:'a@b',zincir:'b2',t:null,yeni:false},{id:'c3',konu:'Eski',kimden:'x@y',zincir:'c3',t:null,yeni:false}]}};await mail(true);
cevap.veri.okunmamis=41;cevap.veri.liste[0].yeni=true;await mail(true);
esit('made unread again in Gmail: bold again',[mailV.liste[0].yeni,q('#mailSay').textContent],[true,'41']);
mailMenu(10,10,mailV.liste[2]);esit('read mail: unread and archive offered',[...q('#menu').querySelectorAll('button')].map(b=>b.textContent),['Open in Gmail','Mark as unread','Archive']);q('#menu').hidePopover?.();
cevap={durum:'İnternete bağlanılamadı'};await mail(true);
esit('error keeps the last list',[q('#mail').querySelectorAll('.hb').length,q('#mailAlt').textContent.endsWith(' · could not refresh')],[3,true]);
cevap={durum:'ok',veri:{okunmamis:0,liste:[]}};await mail(true);
esit('empty inbox',[q('#mail').textContent,q('#mailSay').textContent],['The inbox is empty.','']);

// layout backups
yerListe();esit('no backups yet',[q('#aYerSec').textContent,q('#aYerYukle').disabled],['No saved layouts',true]);
window.sor=async()=>['Çalışma düzeni'];
S.ayar.kartBoy={takvim:2};S.ayar.yerlesim={sol:['saat'],orta:['takvim'],sag:['hava']};await q('#aYerKaydet').onclick();
esit('backup saved',S.ayar.yerYedek.map(y=>[y.ad,Object.keys(y.v)]),[['Çalışma düzeni',['yerlesim','kartBoy','kartKapali']]]);
S.ayar.kartBoy={takvim:.5};S.ayar.yerlesim={sol:['hava'],orta:['saat'],sag:['takvim']};S.ayar.yanOran={'a|b':.6};
q('#aYerSec').value='0';q('#aYerYukle').onclick();await bek(100);
esit('backup applied',durum&&[durum.ayar.kartBoy,durum.ayar.yerlesim,durum.ayar.yanOran,durum.ayar.yerYedek.map(y=>y.ad)],
  [{takvim:2},{sol:['saat'],orta:['takvim'],sag:['hava']},undefined,['Previous layout (automatic)','Çalışma düzeni']]);
esit('previous layout kept',durum?.ayar.yerYedek[0].v.yanOran,{'a|b':.6});
S.ayar.yerYedek=[{ad:'bozuk'},null,{ad:'iyi',t:1,v:{}}];esit('broken backups are skipped',yerListe().length,1);
