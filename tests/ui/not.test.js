// dil: tr
// Notes: clear checked items (with what is nested under them), copy, move tabs, the list redrawn only when the first line changes
const {esit,dogru,q,bek}=RT;
for(const x of document.querySelectorAll('dialog[open]'))x.close();
esit('checked items and their sub-lines go',isaretliSil('Alışveriş\n- [x] süt\n  - yarım litre\n- [ ] ekmek\n  - [x] tam buğday\n- [X] yumurta\nson'),'Alışveriş\n- [ ] ekmek\nson');
S.notlar=[{ad:'A',metin:'- [x] bitti\n- [ ] kaldı'},{ad:'B',metin:'b'},{ad:'C',metin:'c'}];S.aktifNot=0;notCiz();
const sec=t=>[...q('#menu').querySelectorAll('button')].find(b=>b.textContent===t);
notMenu(10,10,0);await bek(20);sec('İşaretlenenleri temizle').click();
esit('cleared, shown in the editor',[S.notlar[0].metin,q('#not').value],['- [ ] kaldı','- [ ] kaldı']);
q('#tost button').click();esit('undo',S.notlar[0].metin,'- [x] bitti\n- [ ] kaldı');
notMenu(10,10,1);await bek(20);dogru('no checked items: not offered',!sec('İşaretlenenleri temizle'));
sec('Sağa taşı').click();esit('moved right, active note kept',[S.notlar.map(n=>n.ad),S.notlar[S.aktifNot].ad],[['A','C','B'],'A']);
// typing: the list is redrawn only when the first line changes
const ilk=q('#nliste').innerHTML;let n=0;const ob=new MutationObserver(()=>n++);ob.observe(q('#nliste'),{childList:true,subtree:true});
const ta=q('#not');ta.value=S.notlar[0].metin+'\nyeni satır';ta.dispatchEvent(new Event('input'));await bek(20);
esit('later line typed: list not redrawn',n,0);
ta.value='Yeni başlık\n'+ta.value;ta.dispatchEvent(new Event('input'));await bek(20);
dogru('first line typed: list updated',n>0&&q('#nliste').textContent.includes('Yeni başlık'));
