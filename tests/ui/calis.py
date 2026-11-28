"""Headless UI tests for src/ui.html.

Builds the page with stubbed /api answers and demo data, then runs every tests/ui/*.test.js in headless Edge.
Usage:  python tests/ui/calis.py [name ...]     (no names: all tests; the JS syntax is checked with node first)

A test file is the body of an async function. Helpers on window.RT:
  RT.esit(name, actual, expected)   deep equality through JSON
  RT.dogru(name, value)             truthy
  RT.bek(ms), RT.q(selector), RT.FIX (tests/fixtures/*.json by file name without .json)
First-line options:  // dil: tr|en|pl|es, or several as pl,es (runs once per language; default en)     // sure: <virtual ms>   (default 15000)
Window errors fail the test. Exit code 1 when anything fails.
"""
import datetime as dt
import glob
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

KOK = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BURA = os.path.join(KOK, 'tests', 'ui')
EDGE = next((p for p in (os.path.expandvars(r'%ProgramFiles(x86)%\Microsoft\Edge\Application\msedge.exe'),
                         os.path.expandvars(r'%ProgramFiles%\Microsoft\Edge\Application\msedge.exe')) if os.path.exists(p)), None)


def demo():
    """Demo answers of the stubbed server (no personal data)."""
    n = dt.datetime.now()
    g = lambda d=0: (n + dt.timedelta(days=d)).strftime('%Y-%m-%d')
    t45 = (n + dt.timedelta(minutes=45)).strftime('%Y-%m-%dT%H:%M:00+03:00')
    t105 = (n + dt.timedelta(minutes=105)).strftime('%Y-%m-%dT%H:%M:00+03:00')
    ev = [{"id": "m", "summary": "Danışman toplantısı", "start": {"dateTime": t45}, "end": {"dateTime": t105}, "_t": "primary"},
          {"id": "f", "summary": "NÖHÜ · Güz dönemi final sınavları", "start": {"date": "2027-01-04"}, "end": {"date": "2027-01-13"}, "_t": "primary", "extendedProperties": {"private": {"ryocs": "onemli"}}},
          {"id": "y", "summary": "Yılbaşı", "start": {"date": "2027-01-01"}, "end": {"date": "2027-01-02"}, "_t": "tr.turkish#holiday@group.v.calendar.google.com"}]
    hv = {"current": {"time": n.strftime('%Y-%m-%dT%H:00'), "temperature_2m": 17.2, "apparent_temperature": 15, "weather_code": 2, "wind_speed_10m": 10, "wind_direction_10m": 220, "relative_humidity_2m": 71, "pressure_msl": 1014.6, "precipitation": 0.2},
          "hourly": {"time": [(n + dt.timedelta(hours=h)).strftime('%Y-%m-%dT%H:00') for h in range(30)], "temperature_2m": [18 - abs(h - 10) * .6 for h in range(30)], "precipitation_probability": [max(0, (h - 12) * 6) for h in range(30)], "weather_code": [2] * 30},
          "daily": {"time": [g(i) for i in range(7)], "weather_code": [2, 3, 61, 2, 0, 1, 80], "temperature_2m_max": [18, 18, 14, 15, 19, 20, 13], "temperature_2m_min": [9, 9, 11, 8, 7, 10, 6], "precipitation_probability_max": [10, 20, 60, 10, 0, 5, 70], "uv_index_max": [3.4, 2, 1, 4, 5, 5, 2],
                    "sunrise": [g() + 'T06:58'] * 7, "sunset": [g() + 'T16:52'] * 7}, "sehir": "Köyceğiz", "bolge": "Muğla"}
    st = {"tema": "", "ayar": {"cam": True}, "odak": {g(): 3, g(-1): 5, g(-2): 2}, "biten": {g(): 1, g(-3): 4},
          "links": [{"a": "Posta", "u": "https://mail.google.com"}],
          "todos": [{"t": "Rapor", "due": g() + "T18:00"}, {"t": "Ödev", "due": g() + "T20:00"}],
          "uygulamalar": [{"ad": "Notepad", "yol": "C:/Windows/notepad.exe", "simge": "", "n": 3, "son": int(n.timestamp() * 1000) - 600000},
                          {"ad": "Hunt Showdown 1896", "yol": "steam://rungameid/594650", "simge": ""}]}
    ks = [{"ad": "GIMP 3.2.4", "yol": "C:/g.lnk"}, {"ad": "Tetris® Effect Connected", "yol": "C:/t.url"}]
    hb = [{"title": b, "link": "https://example.com/" + str(i), "channelName": c, "channel_logo_src": "", "imageLink": "", "pubDate": (n - dt.timedelta(minutes=7 * i + 3)).strftime("%Y-%m-%dT%H:%M:%S")}
          for i, (b, c) in enumerate([("Merkez Bankası faiz kararını açıkladı", "Gazete Oksijen"), ("Fenerbahçe deplasmanda kazandı", "beIN SPORTS"),
                                      ("Yapay zekâ düzenlemesi Avrupa Parlamentosundan geçti", "BBC Türkçe"), ("Borsa güne yükselişle başladı", "Ekonomist")] * 2)]
    return st, hv, ev, ks, hb


def js(v):
    return json.dumps(v, ensure_ascii=False).replace('</', '<\\/')


def doldur(kalip, *degerler):
    """Fills %s slots in order (not the % operator: tests and data may contain % signs)."""
    p = kalip.split('%s')
    assert len(p) == len(degerler) + 1
    return ''.join(x + y for x, y in zip(p, list(degerler) + ['']))


def sayfa(test, dil):
    s = open(os.path.join(KOK, 'src', 'ui.html'), encoding='utf-8').read()
    # the dictionaries go in as the server does (src/main.rs soz_js)
    soz = {d: json.load(open(os.path.join(KOK, 'src', 'dil', d + '.json'), encoding='utf-8')) for d in ('pl', 'es')}
    s = s.replace('__SOZ__', js(soz), 1)
    st, hv, ev, ks, hb = demo()
    fix = {os.path.basename(p)[:-5]: json.load(open(p, encoding='utf-8')) for p in glob.glob(os.path.join(KOK, 'tests', 'fixtures', '*.json'))}
    on = ("<script>window.HATA=[];addEventListener('error',e=>HATA.push(e.message+' @'+e.lineno));"
          f"try{{localStorage.dil='{dil}'}}catch{{}}"
          "window.fetch=async p=>({json:async()=>p==='/api/state'?{durum:'ok',veri:%s}:p.split('?')[0]==='/api/weather'?{durum:'ok',veri:%s}:p.startsWith('/api/calendar')?{durum:'ok',veri:{items:%s,takvimler:[{id:'primary',ad:'Ana',renk:'#8fb996',yaz:true}]}}:p==='/api/kisayollar'?{durum:'ok',veri:%s}:p==='/api/simgeler'?{durum:'ok',veri:{}}:p==='/api/haber'?{durum:'ok',veri:%s}:p==='/api/uygulama'?{durum:'ok'}:{durum:'kimlik_yok'}});"
          "window.RT={ok:0,fail:[],FIX:%s,"
          "esit(a,g,b){const x=JSON.stringify(g),y=JSON.stringify(b);if(x===y)this.ok++;else this.fail.push(a+': '+x+' != '+y)},"
          "dogru(a,v){if(v)this.ok++;else this.fail.push(a)},bek:ms=>new Promise(r=>setTimeout(r,ms)),q:s=>document.querySelector(s)};"
          "RT.esit=RT.esit.bind(RT);RT.dogru=RT.dogru.bind(RT);</script>")
    on = doldur(on, js(st), js(hv), js(ev), js(ks), js(hb), js(fix))
    son = ("<script>setTimeout(async()=>{try{\n%s\n}catch(e){RT.fail.push('exception: '+(e.stack||e))}"
           "const h=HATA.filter(m=>!/ResizeObserver loop/.test(m));for(const m of h)RT.fail.push('window error: '+m);"
           "const p=document.createElement('pre');p.id='SONUC';p.textContent=JSON.stringify({ok:RT.ok,fail:RT.fail});document.body.append(p)},2500)</script>")
    son = doldur(son, test)
    i = s.index('<script')
    return s[:i] + on + s[i:].replace('</body>', son + '</body>', 1)


def calistir(yol, dil):
    test = open(yol, encoding='utf-8').read()
    secenek = dict(re.findall(r'^//\s*(dil|sure):\s*(\S+)', test, re.M))
    gecici = tempfile.mkdtemp(prefix='ryocs-ui-')
    try:
        html = os.path.join(gecici, 'sayfa.html')
        open(html, 'w', encoding='utf-8').write(sayfa(test, dil))
        r = subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--no-first-run', '--lang=' + ('tr-TR' if dil == 'tr' else 'en-US'),
                            '--user-data-dir=' + os.path.join(gecici, 'profil'), '--window-size=1456,819',
                            '--virtual-time-budget=' + secenek.get('sure', '15000'), '--dump-dom', 'file:///' + html.replace('\\', '/')],
                           capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=180)
        m = re.search(r'<pre id="SONUC">(.*?)</pre>', r.stdout, re.S)
        if not m:
            return 0, ['no result (the test did not finish within the virtual time budget, or the page crashed)']
        v = json.loads(m.group(1).replace('&amp;', '&').replace('&lt;', '<').replace('&gt;', '>').replace('&quot;', '"'))
        return v['ok'], v['fail']
    finally:
        shutil.rmtree(gecici, ignore_errors=True)


def sozdizimi():
    """node --check on the page's scripts (catches syntax errors the browser would only log)."""
    if not shutil.which('node'):
        print('SKIP js syntax (node not found)')
        return True
    s = open(os.path.join(KOK, 'src', 'ui.html'), encoding='utf-8').read()
    with tempfile.NamedTemporaryFile('w', suffix='.js', delete=False, encoding='utf-8') as f:
        f.write('\n'.join(re.findall(r'<script>(.*?)</script>', s, re.S)))
    r = subprocess.run(['node', '--check', f.name], capture_output=True, text=True)
    os.remove(f.name)
    # a line comment that ran into the next statement silently disables it (the hourly update check and the weather refresh were lost this way)
    yutulan = [m.group(0)[:120] for m in re.finditer(r'(?<![:"\'\\])//[^\n]*?(?:setInterval|setTimeout|addEventListener)\(', s)]
    for y in yutulan:
        print('FAIL code inside a // comment: ' + y)
    print('PASS js syntax' if r.returncode == 0 else 'FAIL js syntax\n' + r.stderr)
    return r.returncode == 0 and not yutulan


def main():
    if not EDGE:
        print('SKIP ui tests: Microsoft Edge not found')
        return 0
    ok = sozdizimi()
    sys.path.insert(0, os.path.join(KOK, 'tools'))
    import ceviri
    ok = ceviri.denetle() and ok
    adlar = sys.argv[1:]
    for yol in sorted(glob.glob(os.path.join(BURA, '*.test.js'))):
        ad = os.path.basename(yol)[:-8]
        if adlar and ad not in adlar:
            continue
        diller = (re.search(r'^//\s*dil:\s*(\S+)', open(yol, encoding='utf-8').read(), re.M) or [0, 'en'])[1].split(',')
        for dil in diller:
            n, hatalar = calistir(yol, dil)
            print(('PASS' if not hatalar else 'FAIL') + f' {ad}' + (f' [{dil}]' if len(diller) > 1 else '') + f' ({n} checks)')
            for h in hatalar:
                print('   - ' + h)
            ok = ok and not hatalar
    return 0 if ok else 1


if __name__ == '__main__':
    sys.exit(main())
