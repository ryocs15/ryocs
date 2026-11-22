// dil: en
// data.json format: saves from 0.2–0.4 open through normal() with defaults and migrations; the format number is set
const {esit,dogru}=RT;
const kopya=x=>JSON.parse(JSON.stringify(x));
esit('format number',VERI_SURUM,1);

// 0.2: a single note string, no cards list, old focus days
const a=normal(kopya(RT.FIX['data-0.2']));
esit('0.2: note migrated',[a.notlar,'notes' in a],[[{ad:'Note',metin:'single old note\n- [ ] item'}],false]);
esit('0.2: second clock kept as before',a.ayar.saat2,'Europe/Warsaw');
esit('0.2: later cards start closed',[a.ayar.kartKapali,a.ayar.kartBilinen],[['kur','sistem','pano','mail','ekran'],['kur','sistem','pano','mail','ekran']]);
esit('0.2: days older than 90 removed',[a.odak,a.biten],[{},{}]);
esit('0.2: defaults',[a.uygulamalar,a.aliskanlik,a.aktifNot,a.surum,a.todos.length],[[],[],0,1,2]);

// 0.3: several notes, a card added later (mail), an unknown field
const b=normal(kopya(RT.FIX['data-0.3']));
esit('0.3: note index clamped',b.aktifNot,1);
esit('0.3: second clock off stays off',b.ayar.saat2,'');
esit('0.3: only the new card closes',[b.ayar.kartKapali,b.ayar.kartBilinen],[['pano','mail','ekran'],['kur','sistem','pano','mail','ekran']]);
esit('0.3: unknown fields kept',b.gelecekAlan,'kept as is');
esit('0.3: layout untouched',b.ayar.yerlesim.orta,['gorevler','notlar']);

// 0.4: extra weather cities as names and as places, plus broken entries
const c=normal(kopya(RT.FIX['data-0.4']));
esit('0.4: nothing reopened, only the screen time card closes',[c.ayar.kartKapali,c.ayar.saat2],[['ekran'],'Asia/Tokyo']);
const eski=S;S=c;
esit('0.4: city list read safely',sehirler(),[{ad:'Ankara'},{ad:'Yeşilova',bolge:'Burdur',lat:37.5,lon:29.75}]);
S=eski;

// a fresh install and a save from a newer RYOCS
const d=normal({});
esit('fresh install: second clock off, format set',[d.ayar.saat2,d.surum,d.notlar.length],[undefined,1,1]);
esit('newer save keeps its number',normal({surum:7,ayar:{x:1}}).surum,7);
dogru('normal() is repeatable',JSON.stringify(normal(kopya(a)))===JSON.stringify(a));
