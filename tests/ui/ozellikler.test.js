// dil: en
// Second clock cities, focus labels and note, update check (v0.4 features)
const {esit,dogru,q,bek}=RT;
const f0=window.fetch;let acilan=null;
window.fetch=async(p,a)=>{
  if(p==='/api/guncelleme')return{json:async()=>({durum:'ok',veri:{surum:'0.4.2',son:'0.5.0',yeni:true,url:'https://github.com/ryocs15/ryocs/releases/tag/v0.5.0'}})};
  if(p==='/api/open'){acilan=JSON.parse(a.body).url;return{json:async()=>({durum:'ok'})}}
  if(p==='/api/state'&&a?.method==='POST')return{json:async()=>({durum:'ok'})};
  return f0(p,a)};

// second clock: every zone must be known to the browser (an unknown one would throw every second)
esit('second clock: number of choices',Object.keys(SAAT2).length,37);
let bilinmeyen=[];for(const z of Object.keys(SAAT2))if(z)try{dilimSaat(z,new Date())}catch{bilinmeyen.push(z)}
esit('second clock: all zones known',bilinmeyen,[]);
S.ayar.saat2='Asia/Tokyo';ustSon='';ustCiz();
dogru('second clock shows Tokyo',q('#saat2').textContent.startsWith('Tokyo '));

// focus: own labels and the "working on" note
esit('focus: default label',q('#mod').textContent,'Work');
S.ayar.calismaAd='Study';S.ayar.molaAd='Tea';odakCiz();
esit('focus: own label',q('#mod').textContent,'Study');
esit('focus: note placeholder',[q('#odakNe').textContent,q('#odakNe').classList.contains('bos')],['+ What are you working on?',true]);
S.ayar.odakNe='Math homework';odakCiz();
esit('focus: note shown',[q('#odakNe').textContent,q('#odakNe').classList.contains('bos')],['Math homework',false]);
esit('focus: round end texts',[odakMetin('mola'),odakMetin('calisma')],['Study done (Math homework). Tea: 5 min.','Tea over. Next: Study (Math homework).']);
S.ayar.calismaAd='   ';odakCiz();esit('focus: blank label falls back',q('#mod').textContent,'Work');

// update check
await guncelle(true);
esit('update: settings row',[q('#aGunD').textContent,q('#aGunB').textContent],['new version available: 0.5.0','Download']);
dogru('update: toast',q('#tost').textContent.startsWith('A new version is available: RYOCS 0.5.0'));
q('#aGunB').click();await bek(50);
esit('update: download opens the release page',acilan,'https://github.com/ryocs15/ryocs/releases/tag/v0.5.0');
