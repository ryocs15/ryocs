// dil: tr
// Developer options: off by default hides the tools and their Ctrl+F commands; the switch shows both and is saved
const {esit,dogru,q}=RT;
const ara=()=>komutlar().map(k=>k[0]);
ayarCiz();
dogru('off by default: tools hidden',q('#aGelisAlan').hidden);
dogru('off by default: no diagnostic commands',!ara().includes('Kare hızını ölç'));
q('#aGelis').click();
esit('switch saved',S.ayar.gelistirici,true);
dogru('on: tools shown',!q('#aGelisAlan').hidden);
dogru('on: commands in Ctrl+F',ara().includes('Kare hızını ölç')&&ara().includes('Yerleşimi denetle'));
q('#aGelis').click();
dogru('off again: hidden',q('#aGelisAlan').hidden&&!ara().includes('Yerleşimi denetle'));
