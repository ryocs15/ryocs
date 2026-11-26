// sure: 30000
// A card's hand-set minimum height (kartMin) below its content: when the middle column overflows, the card below must not overlap it
const {esit,bek,q}=RT;
const yerl={"orta":["odak","uygulamalar","saat","takvim","notlar"],"sag":["gorevler","sistem","haber","mail","pano","kur","ekran"],"sol":["spotify","hava"]};
for(const ky of ['icerik',undefined])for(const m of [300,400]){
  Object.assign(S.ayar,{yerlesim:yerl,kartMin:{takvim:m}});if(ky)S.ayar.kartYuk=ky;else delete S.ayar.kartYuk;
  yerlesimUygula();await bek(300);
  const a=q('[data-kart=takvim]').getBoundingClientRect(),b=q('[data-kart=notlar]').getBoundingClientRect();
  esit(`${ky||'normal'} min${m}: the card below starts after the calendar`,b.top>=a.bottom,true);
}
