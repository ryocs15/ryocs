// dil: en
// sure: 30000
// No Turkish text in the English UI: opens every card, card settings, every settings category, search, menus and the countdown list.
// Turkish demo data (event names, the city) and the "Türkçe" language option are allowed.
const {esit,q,bek}=RT;
const IZIN=['Danışman toplantısı','NÖHÜ · Güz dönemi final sınavları','Yılbaşı','Ödev','Rapor','Köyceğiz','Muğla','Posta','Türkçe','uygulama yeniden yüklenir / the app reloads',
  'Merkez Bankası faiz kararını açıkladı','Fenerbahçe deplasmanda kazandı','Yapay zekâ düzenlemesi Avrupa Parlamentosundan geçti','Borsa güne yükselişle başladı','BBC Türkçe','Ekonomist','Gazete Oksijen','beIN SPORTS'];
const harf=s=>s.replace(/[^\p{L}\p{N}\s]/gu,' ').replace(/\s+/g,' ').trim();
const izinli=t=>{const c=harf(t.replace(/^@[\w-]+: /,''));return IZIN.some(d=>{const h=harf(d);return h.includes(c)||c.includes(h)})};
const TRW=/[çğıöşüÇĞİÖŞÜ]|\b(ve|ile|için|yok|bir|gün|dk|sa|kez|tur|görev|etkinlik|hissedilen|Kaydet|Ekle|Sil|Kapat|Aç|Bugün|Yarın)\b/;
const bul=new Map();
const tara=etiket=>{const w=document.createTreeWalker(document.body,NodeFilter.SHOW_TEXT);
  for(let n;n=w.nextNode();){const t=n.nodeValue.trim(),e=n.parentElement;if(!t||!TRW.test(t)||e.closest('script,style,pre'))continue;if(e.closest('[hidden]')&&!e.closest('dialog[open]'))continue;if(!izinli(t))bul.set(t.slice(0,90),etiket)}
  for(const e of document.querySelectorAll('[title],[aria-label],[placeholder]'))for(const k of ['title','aria-label','placeholder']){const t=e.getAttribute(k);if(t&&TRW.test(t)&&!izinli(t))bul.set('@'+k+': '+t.slice(0,90),etiket)}};
for(const k of ['kur','sistem','pano','mail'])kartAc(k,true);await bek(300);tara('main');
document.querySelectorAll('main [data-kart]').forEach(k=>k.classList.add('genis'));
for(const k of Object.keys(KART_AD)){kartAyar(k);await bek(60);tara('card settings: '+k);q('dialog[open] [data-kapat]')?.click();await bek(30)}
ayarAc();await bek(400);
for(const b of document.querySelectorAll('.ayNav button')){b.click();await bek(80);tara('settings: '+b.textContent)}
document.querySelectorAll('dialog[open]').forEach(d=>d.close());
const ara=q('#ara');ara.focus();
for(const v of ['a','zzqx']){ara.value=v;ara.dispatchEvent(new Event('input',{bubbles:true}));await bek(200);tara('search '+v)}
ara.value='';ara.dispatchEvent(new Event('input',{bubbles:true}));
for(const k of Object.keys(KART_AD)){const el=q(`[data-kart="${k}"]`);if(!el)continue;const r=el.getBoundingClientRect();
  el.dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,cancelable:true,clientX:r.left+5,clientY:r.top+5}));await bek(40);tara('menu: '+k);q('#menu').hidePopover?.()}
sayimPencere();await bek(80);tara('countdowns');document.querySelectorAll('dialog[open]').forEach(d=>d.close());
esit('Turkish text in the English UI',[...bul].map(([t,e])=>e+' | '+t),[]);
