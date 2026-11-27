// dil: en
// Deleting a note keeps the open one open (B stays open when A on its left is deleted); the clock card's date line reads in order
const {esit,q,bek}=RT;
S.notlar=[{ad:'A',metin:'a'},{ad:'B',metin:'b'},{ad:'C',metin:'c'}];S.aktifNot=1;notCiz();
notMenu(0,0,0);await bek(30);[...document.querySelectorAll('.menu button')].find(b=>b.textContent.trim()==='Delete').click();
esit('open note after deleting the one on its left',[S.notlar.map(n=>n.ad).join(''),S.notlar[S.aktifNot].ad],['BC','B']);
esit('date line',Lf('{0}. hafta · yılın {1}. günü · ','Week {0} · day {1} of the year · ',47,331),'Week 47 · day 331 of the year · ');
