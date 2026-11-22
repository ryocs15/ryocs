// dil: tr
// Free timer from the clock card: start, pause / resume (remaining time frozen, kept across a restart), one notification at the end;
// stopwatch: runs, pauses, stops
const {esit,dogru,q,bek}=RT;
const bildirim=[],f0=window.fetch;
window.fetch=async(p,a)=>{if(p==='/api/notify'){bildirim.push(JSON.parse(a.body));return{json:async()=>({durum:'ok'})}}return f0(p,a)};
for(const x of document.querySelectorAll('dialog[open]'))x.close();
SY=null;sySakla();KR=null;krSakla();serbestCiz();
esit('nothing running: clock line empty',q('#saatSay').textContent,'');
// started from the clock card's menu
saatMenu(10,10);await bek(20);
[...q('#menu').querySelectorAll('button')].find(b=>b.textContent==='Sayaç başlat…').click();await bek(60);
q('#sor input[name=a0]').value='Çay';q('#sor input[name=a1]').value='5';q('#sor form').requestSubmit();await bek(60);
dogru('timer on the clock card',q('#saatSay').textContent.startsWith('⏳ Çay · 05:00')||q('#saatSay').textContent.startsWith('⏳ Çay · 04:59'));
dogru('and on the focus card',q('#serbest').textContent.includes('Çay'));
// pause: the remaining time stays, it survives a reload of the saved state
q('#saatSay [data-ss="sy"]').click();
const k1=syKalan();await bek(1200);serbestCiz();
esit('paused: time frozen',syKalan(),k1);
dogru('paused look',q('#saatSay b').classList.contains('durdu'));
dogru('saved as paused',JSON.parse(localStorage.sayac).dur>0);
q('#serbest [data-sy="d"]').click();
dogru('resumed from the focus card',!(SY.dur>0)&&SY.bitis>Date.now());
// end: one notification
SY.bitis=Date.now()-1;serbestCiz();serbestCiz();await bek(30);
esit('one notification',bildirim.filter(b=>b.baslik==='Sayaç').length,1);
esit('timer gone',[SY,q('#saatSay').textContent],[null,'']);
// stopwatch
krBaslat();KR.bas-=65e3;saatSayCiz();
dogru('stopwatch runs',q('#saatSay').textContent.startsWith('⏱ 01:05'));
q('#saatSay [data-ss="kr"]').click();const m1=krMs();await bek(1100);
esit('stopwatch paused',krMs(),m1);
q('#saatSay [data-ss="kr"]').click();await bek(1100);
dogru('resumed',krMs()>m1+900);
dogru('menu offers stop',(saatMenu(10,10),[...q('#menu').querySelectorAll('button')].some(b=>b.textContent==='Kronometreyi durdur')));q('#menu').hidePopover();
q('#saatSay [data-ss="krx"]').click();
esit('stopped',[KR,localStorage.kronometre,q('#saatSay').textContent],[null,undefined,'']);
window.fetch=f0;
