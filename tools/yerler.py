"""Builds data/yerler.tsv (country -> province -> district list for the place picker) from GeoNames dumps.

Usage:  python tools/yerler.py <folder with admin1CodesASCII.txt and admin2Codes.txt>
        (https://download.geonames.org/export/dump/, CC BY 4.0)
Row:    country<TAB>province code<TAB>district code (empty for a province)<TAB>geonameid<TAB>name
Coordinates are not stored: the app looks the chosen place up by its geonameid.
"""
import os
import sys

KOK = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EKLER = (' İlçesi', ' District')


def ad(s):
    s = s.strip()
    for e in EKLER:
        if s.endswith(e):
            s = s[:-len(e)]
    return s.replace('\t', ' ')


def main(klasor):
    satir = []
    for dosya, parca in (('admin1CodesASCII.txt', 2), ('admin2Codes.txt', 3)):
        for l in open(os.path.join(klasor, dosya), encoding='utf-8'):
            k = l.rstrip('\n').split('\t')
            if len(k) < 4 or not k[3].isdigit():
                continue
            kod = k[0].split('.')
            if len(kod) != parca or not ad(k[1]):
                continue
            satir.append((kod[0], kod[1], kod[2] if parca == 3 else '', k[3], ad(k[1])))
    satir.sort()
    hedef = os.path.join(KOK, 'data', 'yerler.tsv')
    os.makedirs(os.path.dirname(hedef), exist_ok=True)
    with open(hedef, 'w', encoding='utf-8', newline='\n') as f:
        f.writelines('\t'.join(r) + '\n' for r in satir)
    # self-check: the case that started this (Köyceğiz in Muğla, not the village in Şırnak)
    assert ('TR', '48', '', '304183', 'Muğla') in satir
    assert ('TR', '48', '8631961', '8631961', 'Köyceğiz') in satir
    assert not any(r[4].endswith(EKLER) for r in satir)
    print('OK', len(satir), os.path.getsize(hedef), 'bytes')


if __name__ == '__main__':
    main(sys.argv[1])
