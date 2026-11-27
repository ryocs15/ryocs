"""Polish and Spanish texts: src/dil/pl.json and es.json map each English text of the app to its translation.

The English texts are collected from src/ui.html (the second argument of L, Lf and Ln, the values of the static table T)
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


def tara():
    """(texts, texts after a number, calls the dictionary cannot match)."""
    ui = open(os.path.join(KOK, 'src', 'ui.html'), encoding='utf-8').read()
    rs = open(os.path.join(KOK, 'src', 'main.rs'), encoding='utf-8').read()
    # L('tr','en'), Lf('tr','en',...) (fills {0}, {1}…) and Ln('tr','en',n) (text after the number n); either text in '' or ""
    q = '(?:' + JS + '|' + RS + ')'
    ui_k = re.compile(r'\bL([fn]?)\(' + q + ',' + q + r'[,)]')
    rs_k = re.compile(r'\bl\(\s*' + RS + r',\s*' + RS + r'\s*\)')
    bul = list(ui_k.finditer(ui))
    k = {coz(m[4] or m[5]) for m in bul}
    sayili = {coz(m[4] or m[5]) for m in bul if m[1] == 'n'}
    i = ui.index('const T={') + len('const T=')
    k |= set(json.JSONDecoder().raw_decode(ui, i)[0].values())
    k |= {coz(m[2]) for m in rs_k.finditer(rs)}
    # a call whose text is not one plain literal (a template, a joined string, a raw string) never finds its translation
    def kacan(metin, desen, cagri):
        tamam = {m.start() for m in desen.finditer(metin)}
        return [metin[m.start():m.start() + 70] for m in re.finditer(cagri, metin) if m.start() not in tamam]
    kacak = kacan(ui, ui_k, r'\bL[fn]?\(\s*[\'"`]') + kacan(rs, rs_k, r'\bl\(\s*(?:"|r#)')
    return {x for x in k if x.strip() and re.search('[A-Za-z]', x)}, sayili, kacak


def metinler():
    return tara()[0]


PLURAL = {'pl': ('one', 'few', 'many')}  # Ln texts: the forms Intl.PluralRules gives for whole numbers


def oku(d):
    return json.load(open(os.path.join(KOK, 'src', 'dil', d + '.json'), encoding='utf-8'))


def yaz(d, v):
    with open(os.path.join(KOK, 'src', 'dil', d + '.json'), 'w', encoding='utf-8', newline='\n') as f:
        json.dump(dict(sorted(v.items(), key=lambda x: x[0].lower())), f, ensure_ascii=False, indent=0)
        f.write('\n')


def anahtarlar(d, k, sayili):
    """Every key the language needs: the texts, plus a form per plural category for each text after a number."""
    return k | {f'{x}#{c}' for x in sayili for c in PLURAL.get(d, ())}


def denetle():
    k, sayili, kacak = tara()
    ok = not kacak
    for x in kacak:
        print('FAIL translations: not a plain text, it never gets translated: ' + x.replace('\n', ' '))
    yer = lambda x: sorted(re.findall(r'\{\d\}', x))
    for d in DILLER:
        s, gerek = oku(d), anahtarlar(d, k, sayili)
        eksik = sorted(x for x in gerek if not s.get(x))
        fazla = sorted(x for x in s if x not in gerek)
        bozuk = sorted(x for x in gerek if s.get(x) and yer(x.split('#')[0]) != yer(s[x]))
        if bozuk:
            ok = False
            print(f'FAIL translations {d}: {{0}} placeholders differ in ' + ' | '.join(bozuk[:5]))
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
        s, (k, sayili, _) = oku(a[1]), tara()
        print(json.dumps({x: '' for x in sorted(anahtarlar(a[1], k, sayili)) if not s.get(x)}, ensure_ascii=False, indent=0))
    elif a[:1] == ['temizle']:
        k, sayili, _ = tara()
        for d in DILLER:
            g = anahtarlar(d, k, sayili)
            yaz(d, {x: y for x, y in oku(d).items() if x in g})
    else:
        sys.exit(0 if denetle() else 1)
