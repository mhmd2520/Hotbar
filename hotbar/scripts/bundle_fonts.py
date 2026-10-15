"""Download the overlay's Google Fonts (latin + latin-ext woff2) into hotbar/ui/fonts and write fonts.css."""
import os
import re
import urllib.request
from pathlib import Path

URL = ('https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,500;12..96,700;12..96,800'
       '&family=IBM+Plex+Sans:wght@400;500;600&family=JetBrains+Mono:wght@400;600&family=Press+Start+2P&family=VT323&display=swap')
UA = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36'
UI = Path(__file__).resolve().parent.parent / 'ui'
OUT = UI / 'fonts'
KEEP = {'latin', 'latin-ext'}

css = urllib.request.urlopen(urllib.request.Request(URL, headers={'User-Agent': UA})).read().decode()
blocks = re.findall(r'/\* ([\w-]+) \*/\s*(@font-face \{.*?\})', css, re.S)
os.makedirs(OUT, exist_ok=True)
kept, seen = [], {}
for subset, block in blocks:
    if subset not in KEEP:
        continue
    fam = re.search(r"font-family: '([^']+)'", block)[1]
    weight = re.search(r'font-weight: ([\d ]+);', block)[1].replace(' ', '-')
    src = re.search(r'url\((https://[^)]+\.woff2)\)', block)[1]
    name = seen.get(src) or f"{fam.lower().replace(' ', '-')}-{weight}-{subset}.woff2"
    if src not in seen:
        seen[src] = name
        urllib.request.urlretrieve(src, os.path.join(OUT, name))
    kept.append(f'/* {subset} */\n' + block.replace(src, f'fonts/{name}'))

header = '/* Bundled from Google Fonts (SIL Open Font License) by bundle_fonts.py, so the overlay works offline. */\n'
open(UI / 'fonts.css', 'w', encoding='utf-8').write(header + '\n'.join(kept) + '\n')
total = sum(os.path.getsize(os.path.join(OUT, f)) for f in os.listdir(OUT))
print(len(kept), 'faces,', len(seen), 'files,', total // 1024, 'KB')
