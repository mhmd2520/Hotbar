r"""Copy the overlay layouts, themes, skins and effects out of ..\..\design-explorer.html into ui\.

Run from anywhere:  python scripts\extract_ui.py
"""
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent  # hotbar\
EXPLORER = ROOT.parent / "design-explorer.html"
UI = ROOT / "ui"

src = EXPLORER.read_text(encoding='utf-8')


def between(start, end):
    a = src.index(start)
    b = src.index(end, a)
    return src[a:b]


css = between('/* overlay */', '/* fake terminal after launch */')
# the Button lab is explorer-only
lab_a = css.index('/* Button lab */')
lab_b = css.index('/* open animations')
css = css[:lab_a] + css[lab_b:]

js_parts = [
    between('const SANS = ', '/* ───────── items / filtering ───────── */'),
    between('function fuzzy(q, s) {', 'function visible() {'),
    between('/* ───────── interfaces ───────── */', '/* ───────── stage rendering ───────── */'),
    between('/* ───────── sound ───────── */', '/* ───────── open / close / launch ───────── */'),
]

header = '/* Generated from design-explorer.html by extract_ui.py: layouts, themes, skins, effects. */\n'
(UI / 'overlay.css').write_text(header + css, encoding='utf-8')
(UI / 'skins.js').write_text(header + '\n'.join(js_parts), encoding='utf-8')
print('css', len(css), 'js', sum(map(len, js_parts)))
