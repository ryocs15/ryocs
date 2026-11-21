// dil: tr
// 1.1.5: the moving background pauses after 5 minutes without input and resumes on the next input or when the window is shown
const {esit}=RT;
const uyku=()=>document.documentElement.classList.contains('uyku');
gorunmez=false;bosta=false;sonGirdi=Date.now();bostaDenetle();
esit('active: not idle',bosta,false);
sonGirdi=Date.now()-BOSTA_MS-1000;bostaDenetle();
esit('5 min without input: idle and paused',[bosta,uyku()],[true,true]);
dispatchEvent(new PointerEvent('pointermove'));
esit('mouse moves: awake',bosta,false);
sonGirdi=Date.now()-BOSTA_MS-1000;bostaDenetle();
dispatchEvent(new KeyboardEvent('keydown',{key:'a'}));
esit('key press: awake',bosta,false);
sonGirdi=Date.now()-BOSTA_MS-1000;gorunmez=true;bostaDenetle();
esit('in the tray: not counted as idle (already paused)',bosta,false);
gizlendi(false);
esit('shown again: the idle clock restarts',[gorunmez,bosta,Date.now()-sonGirdi<1000],[false,false,true]);
