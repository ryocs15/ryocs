// dil: tr
// Apps card: drag to reorder, Alt+1…9, open as administrator, remove offered when an app cannot be opened
const {esit,dogru,q,bek}=RT;
const istek=[];let cevap={durum:'ok'};const f0=window.fetch;
window.fetch=async(p,a)=>{if(p==='/api/uygulama'){istek.push(JSON.parse(a.body));return{json:async()=>cevap}}return f0(p,a)};
for(const x of document.querySelectorAll('dialog[open]'))x.close();
S.uygulamalar=[{ad:'Bir',yol:'C:\\a\\bir.exe'},{ad:'İki',yol:'C:\\a\\iki.lnk'},{ad:'Üç',yol:'C:\\a\\uc.exe'}];uygCiz();
const adlar=()=>S.uygulamalar.map(u=>u.ad);
// drag "Üç" onto "Bir": it goes first
const dt=new DataTransfer(),b=i=>q(`#uyglar [data-u="${i}"]`);
b(2).dispatchEvent(new DragEvent('dragstart',{bubbles:true,dataTransfer:dt}));
b(0).dispatchEvent(new DragEvent('dragover',{bubbles:true,cancelable:true,dataTransfer:dt}));
b(0).dispatchEvent(new DragEvent('drop',{bubbles:true,cancelable:true,dataTransfer:dt}));
esit('dragged to the front',adlar(),['Üç','Bir','İki']);
dogru('titles show the shortcut',b(0).title.includes('(Alt+1)'));
// Alt+2 opens the second; typing in a field does not
dispatchEvent(new KeyboardEvent('keydown',{code:'Digit2',key:'2',altKey:true,bubbles:true}));await bek(30);
esit('Alt+2',istek.at(-1),{yol:'C:\\a\\bir.exe',yonetici:false});
istek.length=0;q('#ara').focus();q('#ara').dispatchEvent(new KeyboardEvent('keydown',{code:'Digit1',key:'1',altKey:true,bubbles:true}));await bek(30);
esit('in a text field: nothing',istek.length,0);q('#ara').blur();
// administrator: only for .exe
uygMenu(10,10,0);await bek(20);
const ogeler=()=>[...q('#menu').querySelectorAll('button')].map(x=>x.textContent);
dogru('exe: open as administrator offered',ogeler().includes('Yönetici olarak aç'));
[...q('#menu').querySelectorAll('button')].find(x=>x.textContent==='Yönetici olarak aç').click();await bek(30);
esit('sent as administrator',istek.at(-1),{yol:'C:\\a\\uc.exe',yonetici:true});
uygMenu(10,10,2);await bek(20);dogru('shortcut: not offered',!ogeler().includes('Yönetici olarak aç'));q('#menu').hidePopover();
// cannot be opened: the message offers to remove it
cevap={durum:'Dosya bulunamadı'};await uygAc(1);
dogru('message with remove',q('#tost').textContent.startsWith('Bir açılamadı')&&q('#tost button')?.textContent==='Kaldır');
q('#tost button').click();esit('removed',adlar(),['Üç','İki']);
