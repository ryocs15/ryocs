"""Polish and Spanish texts: src/dil/pl.json and es.json map each English text of the app to its translation.

The English texts are collected from src/ui.html (the second argument of L('tr','en'), the values of the static table T)
and src/main.rs (the second argument of l("tr", "en")).

  python tools/ceviri.py            report missing and unused keys per language (exit 1 if any is missing)
  python tools/ceviri.py eksik pl   print the missing keys of pl as a JSON object with empty values, to translate
  python tools/ceviri.py temizle    drop unused keys from both files
"""
import json
import os
import re
import sys

KOK = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DILLER = ('pl', 'es')
JS = "'((?:[^'\\\\]|\\\\.)*)'"
RS = '"((?:[^"\\\\]|\\\\.)*)"'


def coz(s):
    """A JS or Rust string literal's body as the text it gives at runtime."""
    def kac(m):
        c = m.group(1)
        if c[0] == 'u':
            return chr(int(c[1:], 16)) if len(c) == 5 else chr(int(c[2:-1], 16))
        return {'n': '\n', 't': '\t', 'r': '\r', '0': '\0'}.get(c, c)
    return re.sub(r'\\(u\{[0-9a-fA-F]+\}|u[0-9a-fA-F]{4}|.)', kac, s)


def metinler():
    ui = open(os.path.join(KOK, 'src', 'ui.html'), encoding='utf-8').read()
    rs = open(os.path.join(KOK, 'src', 'main.rs'), encoding='utf-8').read()
    # L('tr','en') and Lf('tr','en',...) (Lf fills {0}, {1}…); either text in '' or ""
    q = '(?:' + JS + '|' + RS + ')'
    k = {coz(m[2] or m[3]) for m in re.findall(r'\bLf?\(' + q + ',' + q + r'[,)]', ui)}
    i = ui.index('const T={') + len('const T=')
    k |= set(json.JSONDecoder().raw_decode(ui, i)[0].values())
    k |= {coz(en) for _, en in re.findall(r'\bl\(\s*' + RS + r',\s*' + RS + r'\s*\)', rs)}
    return {x for x in k if x.strip() and re.search('[A-Za-z]', x)}


def oku(d):
    return json.load(open(os.path.join(KOK, 'src', 'dil', d + '.json'), encoding='utf-8'))


def yaz(d, v):
    with open(os.path.join(KOK, 'src', 'dil', d + '.json'), 'w', encoding='utf-8', newline='\n') as f:
        json.dump(dict(sorted(v.items(), key=lambda x: x[0].lower())), f, ensure_ascii=False, indent=0)
        f.write('\n')


def denetle():
    k, ok = metinler(), True
    for d in DILLER:
        s = oku(d)
        eksik = sorted(x for x in k if not s.get(x))
        fazla = sorted(x for x in s if x not in k)
        if eksik:
            ok = False
            print(f'FAIL translations {d}: {len(eksik)} missing, e.g. ' + ' | '.join(eksik[:5]))
        if fazla:
            print(f'NOTE translations {d}: {len(fazla)} unused (python tools/ceviri.py temizle)')
    if ok:
        print(f'PASS translations ({len(k)} texts in {", ".join(DILLER)})')
    return ok


if __name__ == '__main__':
    a = sys.argv[1:]
    if a[:1] == ['eksik']:
        s = oku(a[1])
        print(json.dumps({x: '' for x in sorted(metinler()) if not s.get(x)}, ensure_ascii=False, indent=0))
    elif a[:1] == ['temizle']:
        k = metinler()
        for d in DILLER:
            yaz(d, {x: y for x, y in oku(d).items() if x in k})
    else:
        sys.exit(0 if denetle() else 1)
