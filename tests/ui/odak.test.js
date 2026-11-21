// dil: tr
// Focus card: a round that ended while RYOCS was closed, long breaks, skip and +5 minutes from the menu
const {esit,dogru,q,bek}=RT;
const bild=[];const f0=window.fetch;
window.fetch=async(p,a)=>{if(p==='/api/notify'){bild.push(JSON.parse(a.body));return{json:async()=>({durum:'ok'})}}return f0(p,a)};
for(const x of document.querySelectorAll('dialog[open]'))x.close();
const bugun=gunStr(new Date());S.ayar.rahatsiz=false;S.ayar.odakOto=false;
// ended while closed: before the data loads nothing moves, then it is counted without a late notification
{const s0=S;S.odak[bugun]=0;S=null;O={mod:'calisma',kalan:0,bitis:Date.now()-36e5};odakCiz();
 esit('data not loaded: round waits',[O.mod,O.bitis>0],['calisma',true]);
 S=s0;bild.length=0;odakCiz();
 esit('then counted, no late notification',[S.odak[bugun],O.mod,bild.length],[1,'mola',0]);}
// a fresh end notifies; the 4th round of the day gets the long break
S.ayar.uzunMola=15;S.ayar.uzunAralik=4;S.odak[bugun]=3;bild.length=0;
O={mod:'calisma',kalan:0,bitis:Date.now()-1000};odakCiz();
esit('4th round: long break',[S.odak[bugun],O.mod,O.uzun,O.kalan,q('#mod').textContent],[4,'mola',1,900,'Uzun mola']);
dogru('notification names it',bild[0]?.metin.includes('Uzun mola: 15 dakika'));
esit('kept for a restart',JSON.parse(localStorage.odakO).uzun,1);
O={mod:'calisma',kalan:0,bitis:Date.now()-1000};odakCiz();
esit('5th round: normal break',[O.mod,O.uzun,O.kalan],['mola',undefined,sure('mola')]);
S.ayar.uzunMola=0;S.odak[bugun]=7;O={mod:'calisma',kalan:0,bitis:Date.now()-1000};odakCiz();
esit('long break off (0)',[O.uzun,O.kalan],[undefined,sure('mola')]);
// menu: skip keeps running, is not counted; +5 minutes
const sec=t=>[...q('#menu').querySelectorAll('button')].find(b=>b.textContent.startsWith(t));
O={mod:'calisma',kalan:sure('calisma'),bitis:Date.now()+sure('calisma')*1000};const n0=S.odak[bugun];
q('.halka').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:30,clientY:30}));await bek(50);
esit('menu items',[...q('#menu').querySelectorAll('button')].map(b=>b.textContent),['Turu atla → Mola','+5 dakika','Sıfırla']);
sec('Turu atla').click();
esit('skipped: break running, not counted',[O.mod,O.bitis>Date.now(),S.odak[bugun]],['mola',true,n0]);
odakMenu(10,10);sec('+5').click();
esit('+5 minutes',Math.round((O.bitis-Date.now())/6e4),sure('mola')/60+5);
O={mod:'calisma',kalan:sure('calisma'),bitis:0};odakCiz();odakMenu(10,10);
esit('nothing to reset: no reset item',[...q('#menu').querySelectorAll('button')].map(b=>b.textContent),['Turu atla → Mola','+5 dakika']);q('#menu').hidePopover();
// a 0 in a number setting stays 0 (long break off)
kartAyar('odak');await bek(300);const inp=q('#sor [data-a="uzunMola"]');
inp.value='0';inp.dispatchEvent(new Event('change',{bubbles:true}));await bek(100);
esit('0 is saved',S.ayar.uzunMola,0);
inp.value='';inp.dispatchEvent(new Event('change',{bubbles:true}));await bek(100);
esit('empty: the value stays',S.ayar.uzunMola,0);
