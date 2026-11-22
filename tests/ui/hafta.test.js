// dil: tr
// Weekly summary: lines from focus, tasks, habits and screen time compared with the week before; closed cards leave their line out;
// the Sunday 20:00 notification comes once, not on the first run, not when turned off, not for an old week
const {esit,dogru,q,bek}=RT;
const bildirim=[],f0=window.fetch;
const bas=pztGun(new Date()),bu=gunListe(bas,7),ob=new Date(bas);ob.setDate(ob.getDate()-7);const once=gunListe(ob,7);
window.fetch=async(p,a)=>{
  if(p==='/api/notify'){bildirim.push(JSON.parse(a.body));return{json:async()=>({durum:'ok'})}}
  if(p==='/api/ekran')return{json:async()=>({durum:'ok',veri:{acik:true,bugun:gunStr(new Date()),gunler:{[bu[0]]:{Code:7200,Steam:3600,Gizli:900},[bu[1]]:{Code:3600,msedge:1800},[once[0]]:{Code:18000}}}})};
  return f0(p,a)};
for(const x of document.querySelectorAll('dialog[open]'))x.close();
S.odak={[bu[0]]:3,[bu[2]]:2,[once[1]]:4};S.odakLog=[{g:bu[0],dk:25,ne:'Sunum'},{g:bu[0],dk:25,ne:'Sunum'},{g:bu[2],dk:25,ne:'Rapor'}];
S.biten={[bu[1]]:4,[once[3]]:6};S.aliskanlik=[{ad:'Spor',g:{[bu[0]]:1,[bu[2]]:1}},{ad:'Okuma',g:Object.fromEntries(bu.map(g=>[g,1]))}];
S.ayar.ekranAd={msedge:'Edge'};S.ayar.ekranGizli=['Gizli'];
for(const k of ['odak','gorevler','ekran'])kartAc(k,true);await bek(100);
await haftaOzet(bas,true);await bek(50);
const sat=()=>[...q('#sor').querySelectorAll('.haftaS')].map(s=>s.textContent);
esit('lines',sat(),[
  'Odak 5 tur · 1 sa 15 dkEn çok: Sunum (50 dk)↑ 1 tur',
  'Görevler 4 tamamlandı↓ 2',
  'Alışkanlıklar Spor 2/7 · Okuma 7/7 🔥',
  'Ekran süresi 4 sa 30 dk · günde ort. 39 dkCode · Steam · Edge↓ 30 dk']);
dogru('date range in the header',q('#sor p.m').textContent.includes('–'));
for(const x of document.querySelectorAll('dialog[open]'))x.close();
kartAc('ekran',false);await bek(50);
await haftaOzet(bas,true);await bek(50);
dogru('closed card: no screen time line',!sat().some(s=>s.startsWith('Ekran')));
for(const x of document.querySelectorAll('dialog[open]'))x.close();
// this week so far: compared with the same days; on Monday the finished week before
await haftaOzet();await bek(50);
const gun=new Date().getDay();
dogru('so far, or last week on Monday / a full week on Sunday',gun===0||gun===1?!q('#sor p.m').textContent.includes('şimdiye kadar'):q('#sor p.m').textContent.includes('şimdiye kadar'));
for(const x of document.querySelectorAll('dialog[open]'))x.close();
// notification: Sunday 20:30 of this week
const pz=new Date(bas);pz.setDate(pz.getDate()+6);pz.setHours(20,30,0,0);
delete S.haftaOzet;haftaBildir(pz);
esit('first run: quiet, but remembered',[bildirim.length,S.haftaOzet],[0,gunStr(pz)]);
S.haftaOzet='eski';haftaBildir(pz);haftaBildir(pz);await bek(30);
esit('once',bildirim.filter(b=>b.baslik==='Haftalık özetin hazır').length,1);
const sali=new Date(pz);sali.setDate(sali.getDate()+2);S.haftaOzet='eski';haftaBildir(sali);await bek(30);
esit('too late: none',bildirim.filter(b=>b.baslik==='Haftalık özetin hazır').length,1);
S.haftaOzet='eski';S.ayar.haftaOzet=false;haftaBildir(pz);await bek(30);
esit('turned off: none',bildirim.filter(b=>b.baslik==='Haftalık özetin hazır').length,1);
delete S.ayar.haftaOzet;
dogru('in the command palette',araHesapla('özet').some(g=>g[1].some(x=>x.ad.replace(/<[^>]+>/g,'')==='Haftalık özet')));
window.fetch=f0;
