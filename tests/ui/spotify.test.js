// dil: tr
// Spotify card: podcasts, seek, device / shuffle / repeat menu, opening in Spotify, a poll right at the track end
const {esit,dogru,q,bek}=RT;
const istek=[];let durum=null;const f0=window.fetch;
const ep={id:'e1',name:'Bölüm 12',duration_ms:180000,images:[{url:'https://i.scdn.co/image/ep'}],show:{name:'Bilim Podcast'},external_urls:{spotify:'https://open.spotify.com/episode/e1'}};
const parca={id:'t1',name:'Şarkı',duration_ms:200000,album:{images:[{url:'https://i.scdn.co/image/al'}]},artists:[{name:'Sanatçı'}],external_urls:{spotify:'https://open.spotify.com/track/t1'}};
durum={is_playing:true,progress_ms:60000,item:ep,shuffle_state:false,repeat_state:'off',device:{name:'Masaüstü',volume_percent:50}};
window.fetch=async(p,a)=>{
  if(p.startsWith('/api/spotify')||p==='/api/open'){istek.push([p,a?.body?JSON.parse(a.body):null]);
    const v=p==='/api/spotify'?durum:p==='/api/spotify/devices'?{devices:[{id:'d1',name:'Masaüstü',is_active:true},{id:'d2',name:'Telefon',is_active:false},{id:'d3',name:'Kısıtlı',is_restricted:true}]}:p==='/api/spotify/queue'?{queue:[]}:null;
    return{json:async()=>({durum:'ok',veri:v})}}
  return f0(p,a)};
for(const x of document.querySelectorAll('dialog[open]'))x.close();
spMesgul=false;await spotify('');await bek(50);
esit('podcast: episode, show, cover',[q('#sp .tr b').textContent,q('#sp .tr span').textContent,q('#sp img')?.getAttribute('src')],['Bölüm 12','Bilim Podcast','https://i.scdn.co/image/ep']);
esit('device line',q('#sp .spCihaz').textContent,'Masaüstü');
// open in Spotify
istek.length=0;q('#sp .tr b').click();await bek(20);
esit('title opens Spotify',istek[0],['/api/open',{url:'https://open.spotify.com/episode/e1'}]);
// seek: click in the middle, then the arrow key
const il=q('#spIlerle'),k=il.getBoundingClientRect();
istek.length=0;il.dispatchEvent(new MouseEvent('click',{bubbles:true,clientX:k.left+k.width/2,clientY:k.top+5}));await bek(450);
const seek=istek.find(x=>x[0]==='/api/spotify/seek');
dogru('click seeks to the middle',seek&&Math.abs(seek[1].ms-90000)<2000);
esit('bar moves at once',Math.round(parseFloat(q('#spBar').style.width)),50);
await bek(700);istek.length=0;spPos={p:90000,d:180000,t:Date.now(),on:false};
for(let i=0;i<5;i++)q('#spIlerle').dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowRight',bubbles:true}));await bek(450);
esit('held arrow key: one request, +50 s',istek.filter(x=>x[0]==='/api/spotify/seek').map(x=>x[1]),[{ms:140000}]);
await bek(700);
// menu: shuffle, repeat, devices (active and restricted ones are left out)
durum={...durum,item:parca};spSon='';await spotify('');await bek(50);
q('#sp .spCihaz').click();await bek(80);
const ogeler=()=>[...q('#menu').querySelectorAll('button')].map(b=>b.textContent);
esit('menu items',ogeler(),['Karışık çal','Listeyi tekrarla','Spotify’da aç','Şurada çal: Telefon']);
istek.length=0;[...q('#menu').querySelectorAll('button')].find(b=>b.textContent==='Şurada çal: Telefon').click();await bek(30);
esit('transfer',istek[0],['/api/spotify/transfer',{id:'d2'}]);
await bek(700);istek.length=0;spMenu(10,10);await bek(80);
[...q('#menu').querySelectorAll('button')].find(b=>b.textContent==='Karışık çal').click();await bek(30);
esit('shuffle on',istek.find(x=>x[0]==='/api/spotify/shuffle')?.[1],{on:true});
await bek(700);durum={...durum,shuffle_state:true,repeat_state:'context'};spSon='';await spotify('');
esit('device line shows shuffle and repeat',q('#sp .spCihaz').textContent,'Masaüstü · karışık · tekrar');
spMenu(10,10);await bek(80);
esit('repeat goes on to the track',ogeler().slice(0,2),['Karışık çalmayı kapat','Bu şarkıyı tekrarla']);
q('#menu').hidePopover();
// right-click: the player opens the same menu, the title bar keeps the card menu
q('#sp .tr span').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:20,clientY:20}));await bek(80);
esit('right-click on the player: player menu',ogeler()[0],'Karışık çalmayı kapat');q('#menu').hidePopover();
q('[data-kart=spotify] h2').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:20,clientY:20}));await bek(80);
esit('right-click on the title: card menu',ogeler()[0],'Spotify ayarları…');q('#menu').hidePopover();
// a refused seek: the real position comes back
{const f1=window.fetch;window.fetch=async(p,a)=>p==='/api/spotify/seek'?{json:async()=>({durum:'403: premium'})}:f1(p,a);
 durum={...durum,is_playing:false,progress_ms:20000};spSon='';await spotify('');spAtla(150000);await bek(700);
 esit('refused seek: bar back',[Math.round(parseFloat(q('#spBar').style.width)),q('#tost').textContent.startsWith('Komut reddedildi')],[10,true]);window.fetch=f1}
// a slow device list: the menu opens without it after 1.5 s
{const f1=window.fetch;window.fetch=async(p,a)=>p==='/api/spotify/devices'?new Promise(()=>{}):f1(p,a);
 const t0=Date.now();await spMenu(10,10);esit('slow devices: menu after 1.5 s, no devices',[Date.now()-t0<2500,ogeler().some(x=>x.startsWith('Şurada çal'))],[true,false]);q('#menu').hidePopover();window.fetch=f1}
// the track ends: one extra poll right after it
// counted by the timer it sets, not by requests: the normal 5 s poll may also land in the wait (flaky in CI)
{const st=window.setTimeout;let n=0;window.setTimeout=(f,ms,...x)=>{if(ms===900)n++;return st(f,ms,...x)};
 spPos={p:199500,d:200000,t:Date.now()-1000,on:true};spIlerle();spIlerle();window.setTimeout=st;
 esit('track end: one extra poll',n,1);}
