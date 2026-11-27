// dil: tr
// Settings rows: a label is never squeezed into a narrow column by the control next to it (it broke "Diğer şehirler" letter by letter);
// checked as is and with the window forced to 400 px (a small or scaled screen)
const {dogru,bek}=RT;
const sikisik=[],bak=ad=>{for(const r of document.querySelectorAll('dialog[open] .ar')){
  const e=r.firstElementChild;if(!e||r.children.length<2||!r.offsetParent)continue;
  const l=e.getBoundingClientRect(),k=r.lastElementChild.getBoundingClientRect();
  const c=e.cloneNode(true);c.style.cssText='position:absolute;white-space:nowrap;width:auto';r.append(c);const tam=c.getBoundingClientRect().width;c.remove();
  if(l.width+1<Math.min(120,tam))sikisik.push(`${ad}: ${e.textContent.trim().slice(0,40)} (${Math.round(l.width)}px)`);
  else if(k.top<l.bottom-1&&k.left<l.right-1)sikisik.push(`${ad}: ${e.textContent.trim().slice(0,40)} overlaps its control`);
  if(k.right>r.getBoundingClientRect().right+1)sikisik.push(`${ad}: ${e.textContent.trim().slice(0,40)} control sticks out`);
}};
for(const gen of ['',400]){
  for(const k of Object.keys(KART_AYAR)){kartAyar(k);const d=document.querySelector('dialog[open]');if(gen)d.style.width=gen+'px';await bek(30);bak(k+gen);d.close()}
  ayarAc();const d=$('#ayarlar');if(gen)d.style.width=gen+200+'px'; // settings has its 200 px category list beside the rows
  await bek(50);
  for(const b of document.querySelectorAll('#ayarlar .ayNav [role=tab]')){b.click();await bek(30);bak('ayar/'+b.dataset.b+gen)}
  d.close();d.style.width='';
}
dogru('no squeezed labels'+(sikisik.length?': '+sikisik.join(' | '):''),!sikisik.length);
