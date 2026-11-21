// dil: tr
// Clock card: alarms (once / every day, not at start, through do-not-disturb), next alarm line, text written only on change
const {esit,dogru,q,bek}=RT;
const bild=[];const f0=window.fetch;
window.fetch=async(p,a)=>{if(p==='/api/notify'){bild.push(JSON.parse(a.body));return{json:async()=>({durum:'ok'})}}return f0(p,a)};
for(const x of document.querySelectorAll('dialog[open]'))x.close();
const an=(h,m)=>{const d=new Date();d.setHours(h,m,0,0);return d};
S.alarmlar=[{s:'07:30',ad:'Kalk',tekrar:true,acik:true},{s:'09:00',ad:'Toplantı',tekrar:false,acik:true},{s:'25:00',ad:'bozuk'},{s:'10:00',ad:'kapalı',acik:false}];
// the first minute seen does not ring (RYOCS started at 07:30)
alarmDk='';alarmBak(an(7,30));esit('start minute: no ring',bild.length,0);
alarmBak(an(7,31));alarmBak(an(9,0));
esit('one-time alarm rings once and turns off',[bild.map(b=>b.metin),S.alarmlar[1].acik],[['Toplantı'],false]);
alarmBak(an(9,0));esit('same minute again: no second ring',bild.length,1);
alarmBak(an(10,0));esit('switched-off alarm: silent',bild.length,1);
alarmBak(an(7,30));esit('daily alarm rings, stays on',[bild.at(-1)?.metin,S.alarmlar[0].acik],['Kalk',true]);
// do-not-disturb during focus does not hold an alarm
S.ayar.rahatsiz=true;O={mod:'calisma',kalan:600,bitis:Date.now()+6e5};bild.length=0;alarmBak(an(7,31));alarmBak(an(7,30));
esit('rings during focus',bild.length,1);O={mod:'calisma',kalan:sure('calisma'),bitis:0};
// next alarm line: only active and valid ones
alarmBilgi();esit('next alarm',[q('#sAlarm').hidden,q('#sAlarm').textContent.startsWith('⏰ ')],[false,true]);
S.alarmlar=[];alarmBilgi();esit('no alarms: line hidden',q('#sAlarm').hidden,true);
// the alarm window adds one
window.sor=async()=>['06:45','Koşu','her'];alarmPencere();await bek(50);q('#sor [data-ae]').click();await bek(80);
esit('added from the window',S.alarmlar,[{s:'06:45',ad:'Koşu',tekrar:true,acik:true}]);
q('#sor [data-aa="0"]').click();await bek(50);esit('switch turns it off',S.alarmlar[0].acik,false);
// the clock texts are not rewritten when unchanged
const ob=new MutationObserver(()=>{});let n=0;const ob2=new MutationObserver(l=>n+=l.length);
tick();await bek(20);ob2.observe(q('#tarih'),{childList:true,characterData:true,subtree:true});ob2.observe(q('#selam'),{childList:true,characterData:true,subtree:true});
tick();tick();await bek(20);esit('no rewrite of the same text',n,0);
esit('date in the locale order',q('#tarih').textContent,new Date().toLocaleDateString('tr-TR',{day:'2-digit',month:'2-digit'}));
