// dil: tr
// Random background: a real pattern, kept while settings change, a new one each time the window hides, never the same twice
const {esit,dogru,q}=RT;
esit('option in the list',q('#aArka option[value=rastgele]')?.textContent,'Rastgele');
S.ayar.arka='rastgele';ayarUygula();
const a=K.dataset.arka;
dogru('a real pattern',ARKALAR.includes(a));
ayarUygula();esit('other settings change: same pattern',K.dataset.arka,a);
let onceki=a,tekrar=0;const gorulen=new Set([a]);
for(let i=0;i<20;i++){i%2?kapandi():gizlendi(true);const y=K.dataset.arka;if(y===onceki)tekrar++;onceki=y;gorulen.add(y);gizlendi(false)}
esit('each hide picks another, never the previous one',tekrar,0);
dogru('several patterns come up',gorulen.size>=5);
let son='';try{son=localStorage.arkaSon}catch{}
esit('the last one is remembered for the next start',son,K.dataset.arka);
esit('the setting stays random',S.ayar.arka,'rastgele');
S.ayar.arka='kar';ayarUygula();gizlendi(true);gizlendi(false);
esit('a chosen pattern does not change',K.dataset.arka,'kar');
delete S.ayar.arka;ayarUygula();
