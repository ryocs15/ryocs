// dil: en
// sure: 25000
// Window in the tray: no per-second redraws and no Spotify polling, but a focus round still ends with its notification
const {esit,dogru,q,bek}=RT;
const f0=window.fetch,istek=[];
window.fetch=async(p,a)=>{istek.push(p);if(p==='/api/notify'||p==='/api/state'&&a?.method==='POST')return{json:async()=>({durum:'ok'})};return f0(p,a)};
kapandi();q('#saat').textContent='x';istek.length=0;
await bek(6500);
esit('hidden: clock not redrawn, Spotify not polled',[q('#saat').textContent,istek.filter(p=>p==='/api/spotify').length],['x',0]);
O={mod:'calisma',kalan:0,bitis:Date.now()-1000};tick();await bek(50);
dogru('hidden: the focus round still ends',O.mod==='mola'&&istek.includes('/api/notify'));
gizlendi(false);await bek(100);
dogru('shown again: redrawn and polled',q('#saat').textContent!=='x'&&istek.includes('/api/spotify'));
