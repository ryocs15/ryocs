// dil: tr
// System card: all fixed drives, a full disk in the warning colour, battery time left, uptime, the CPU line of the last minute
const {esit,dogru,q,bek}=RT;
let cevap=null;const f0=window.fetch;
window.fetch=async(p,a)=>p==='/api/sistem'?{json:async()=>({durum:'ok',veri:cevap})}:f0(p,a);
kartAc('sistem',true);await bek(100);
const G=2**30;
cevap={cpu:12,bellek:{toplam:16*G,bos:6*G},diskler:[{ad:'C:',toplam:500*G,bos:20*G},{ad:'D:',toplam:1000*G,bos:600*G}],pil:{yuzde:64,sarj:false,kalan:8100},acik:3*86400+5*3600};
cpuGecmis=[];cpuSonT=0;await sistem();
const satir=()=>[...q('#sistem').querySelectorAll('.sisS')].map(s=>s.textContent);
esit('rows',satir(),['İşlemci%12','Bellek10 / 16 GB','Disk C:20 GB boş / 500 GB','Disk D:600 GB boş / 1.000 GB','Pil%64 · 2 sa 15 dk kaldı']);
esit('full disk in the warning colour',[...q('#sistem').querySelectorAll('.cizgi')].map(c=>c.classList.contains('dolu')),[false,false,true,false,false]);
esit('uptime',q('#sistem .sisAcik').textContent,'Açık kalma süresi: 3 gün 5 sa');
dogru('one reading: no line yet',!q('#sistem .sisEgri'));
cevap={...cevap,cpu:40};await sistem();
esit('two readings: a line with two points',q('#sistem .sisEgri polyline')?.getAttribute('points').split(' ').length,2);
// a pause (window hidden) clears the line
cpuSonT=Date.now()-6e4;cevap={...cevap,cpu:5};await sistem();
dogru('after a pause: the line starts again',!q('#sistem .sisEgri'));
// charging: no time left; an older answer with one disk still works
cevap={cpu:null,bellek:null,disk:{toplam:100*G,bos:50*G},pil:{yuzde:80,sarj:true,kalan:null}};await sistem();
esit('older answer, charging',satir(),['İşlemciölçülüyor…','Disk50 GB boş / 100 GB','Pil%80 · şarjda']);
