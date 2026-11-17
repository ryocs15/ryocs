// Self-check for the "next time" calculation of repeating tasks. Run: node test_sonraki.js
// The functions are taken from src/ui.html (no separate module), so the tested code is the same as in the app.
const fs = require('fs'), assert = require('assert');
const s = fs.readFileSync(__dirname + '/src/ui.html', 'utf8').replace(/\r\n/g, '\n'); // a Windows checkout may have CRLF
const al = (bas, son) => { const i = s.indexOf(bas); assert(i >= 0, bas); return s.slice(i, s.indexOf(son, i) + son.length); };
const kur = dil => new Function(`const DIL='${dil}',L=(tr,en)=>DIL==='tr'?tr:en,LOC=DIL==='tr'?'tr-TR':'en-US',iki=n=>String(n).padStart(2,'0');` + al('const gunStr=', '\n') + al('const saat24=d=>', '\n') + al('const yerel=', '\n')
  + al("const GUNK=", '\n') + al('const tekGun=', '\n') + al('const tekAd=', '\n') + al('const tekCoz=', '\n') + al('function sonraki(', '\n}\n')
  + ';return {sonraki,tekCoz,tekAd,tekGun}')();
const { sonraki, tekCoz, tekAd, tekGun } = kur('tr'), N = t => new Date(t);
const es = (due, tek, simdi, beklenen) => assert.strictEqual(sonraki(due, tek, N(simdi)), beklenen, `${due} ${tek} @${simdi}`);

// daily, year rollover
es('2026-12-31T09:00', 'gun', '2026-12-31T10:00', '2027-01-01T09:00');
// a task missed for a long time: the first repeat after now
es('2026-11-01T09:00', 'gun', '2026-11-16T08:00', '2026-11-16T09:00');
es('2026-11-01T09:00', 'gun', '2026-11-16T09:00', '2026-11-17T09:00');
// weekdays: Friday → Monday (start of week)
es('2026-11-13T18:30', 'isgunu', '2026-11-13T19:00', '2026-11-16T18:30');
// weekly: year rollover
es('2026-12-28T08:00', 'hafta', '2026-12-28T09:00', '2027-01-04T08:00');
// monthly: month end is clamped (31 Jan → 28 Feb; 29 in a leap year), year rollover
es('2027-01-31T10:00', 'ay', '2027-01-31T11:00', '2027-02-28T10:00');
es('2028-01-31T10:00', 'ay', '2028-01-31T11:00', '2028-02-29T10:00');
es('2026-12-15T10:00', 'ay', '2026-12-15T11:00', '2027-01-15T10:00');
// specific days: Mon, Wed, Fri (16 Nov 2026 is a Monday)
es('2026-11-16T07:00', 'g:1,3,5', '2026-11-16T08:00', '2026-11-18T07:00');
es('2026-11-20T07:00', 'g:1,3,5', '2026-11-20T08:00', '2026-11-23T07:00'); // Friday → Monday
es('2026-12-30T07:00', 'g:5', '2026-12-30T08:00', '2027-01-01T07:00');     // year rollover, 1 Jan 2027 is a Friday
// untimed task: tomorrow if today's 09:00 has passed
es('', 'gun', '2026-11-16T12:00', '2026-11-17T09:00');
// parsing day names (Turkish case rules) and a broken rule
assert.strictEqual(tekCoz('Pzt, ÇAR cuma'), 'g:1,3,5');
assert.strictEqual(tekCoz('pazartesi; Pazar'), 'g:0,1');
assert.strictEqual(tekCoz('xyz, pa'), '');
assert.strictEqual(tekCoz('Cumartesi, cum'), 'g:5,6');
assert.strictEqual(tekAd('g:0,1,5'), 'Her Pzt, Cum, Paz');
assert.strictEqual(tekGun('g:9'), null);
assert.strictEqual(tekGun('hafta'), null);
// English day names
const en = kur('en');
assert.strictEqual(en.tekCoz('mon, Wed fri'), 'g:1,3,5');
assert.strictEqual(en.tekAd('g:0,1,5'), 'Every Mon, Fri, Sun');
console.log('sonraki: all checks passed');
