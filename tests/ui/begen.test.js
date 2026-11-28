// Spotify like button: liked state asked once per track, toggles with the library calls, goes back when Spotify refuses
const {esit,dogru,q,bek}=RT;
const istek=[];let yanit={};const f0=window.fetch;
const parca=(id)=>({id,uri:'spotify:track:'+id,name:'Song '+id,duration_ms:200000,album:{images:[]},artists:[{name:'Artist'}],external_urls:{spotify:'https://open.spotify.com/track/'+id}});
let durum={is_playing:true,progress_ms:1000,item:parca('A1'),shuffle_state:false,repeat_state:'off',device:{name:'PC',volume_percent:50}};
window.fetch=async(p,a)=>{
  if(p.startsWith('/api/spotify')){const b=a?.body?JSON.parse(a.body):null;istek.push([p,b]);
    const c=p.slice(13),v=p==='/api/spotify'?{durum:'ok',veri:durum}:p==='/api/spotify/queue'?{durum:'ok',veri:{queue:[]}}:yanit[c]||{durum:'ok',veri:null};
    return{json:async()=>v}}
  return f0(p,a)};
for(const x of document.querySelectorAll('dialog[open]'))x.close();
const begen=()=>q('#sp .spBegen')?.getAttribute('aria-pressed'),sor=c=>istek.filter(([p])=>p==='/api/spotify/'+c);
yanit.begen_mi={durum:'ok',veri:[true]};
spMesgul=false;await spotify('');await bek(50);
esit('liked state comes from the library',[begen(),sor('begen_mi').map(x=>x[1])],['true',[{uri:'spotify:track:A1'}]]);
await spotify('');await bek(50);
esit('asked once per track, not on every poll',sor('begen_mi').length,1);
q('#sp .spBegen').click();await bek(50);
esit('click removes it',[begen(),sor('begenme').map(x=>x[1].uri)],['false',['spotify:track:A1']]);
yanit.begen={durum:'izin_yok'};q('#sp .spBegen').click();await bek(50);
dogru('refused: back to not liked, and the user is told to connect again',begen()==='false'&&/connect Spotify again/.test(document.body.innerText));
durum={...durum,item:parca('B2')};yanit.begen_mi={durum:'ok',veri:[false]};await spotify('');await bek(50);
esit('next track asks again',[begen(),sor('begen_mi').at(-1)[1].uri],['false','spotify:track:B2']);
durum={...durum,item:null};await spotify('');await bek(50);
dogru('no button when nothing is playing',!q('#sp .spBegen'));
