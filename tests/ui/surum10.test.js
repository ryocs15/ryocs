// dil: en
// sure: 20000
// 1.0: every background animation shows moving objects, new themes are offered, every settings category has its panel
const {esit,dogru,q,bek}=RT;
S.ayar.anim=true;ayarUygula();
const bos=[];
for(const o of q('#aArka').options){if(o.value==='yok')continue;S.ayar.arka=o.value;ayarUygula();await bek(30);
  const g=[...document.querySelectorAll('#arka i')].filter(i=>getComputedStyle(i).display!=='none'&&getComputedStyle(i).animationName!=='none');
  if(!g.length)bos.push(o.value||'isik')}
esit('every background has animated objects',bos,[]);
S.ayar.arka='lav';ayarUygula();await bek(30);
dogru('lava lamp: blobs start at the bottom and rise',[...document.querySelectorAll('#arka i')].every(i=>getComputedStyle(i).animationName==='lav'&&parseFloat(getComputedStyle(i).top)>innerHeight*.5));
ayarAc();await bek(200);
dogru('new themes in the picker',['nane','seftali','okyanus','erik'].every(v=>q(`#aTemaDiger [data-v="${v}"], #aTemaDiger [data-t="${v}"]`)||q('#aTemaDiger').innerHTML.includes(TEMALAR[v].ad)));
const kat=[...document.querySelectorAll('.ayNav [data-b]')].map(b=>b.dataset.b);
esit('categories',kat,['gorunum','kartlar','bolge','takvim','odak','profil','genel','hesaplar','veri']);
esit('each category has one panel',kat.filter(b=>document.querySelectorAll(`.ayBol[data-b="${b}"]`).length!==1),[]);
q('.ayNav [data-b="bolge"]').click();await bek(50);
esit('language and region',[q('#ayBas').textContent.trim(),!!q('#aDil').offsetParent,!!q('#aSaat2').offsetParent,!!q('#aCal').offsetParent],['Language and region',true,true,false]);
q('#ayarlar [data-kapat]').click();await bek(300);
q('#avatar').click();await bek(200);
esit('avatar opens the profile',[q('#ayBas').textContent.trim(),!!q('#aAd').offsetParent],['Profile',true]);
