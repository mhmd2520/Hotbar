r"""Write the updater manifest (latest.json) for a signed installer, next to it.

    python scripts\latest_json.py src-tauri\target\release\bundle\nsis\Hotbar_0.1.6_x64-setup.exe
    python scripts\latest_json.py <setup.exe> --url-base http://127.0.0.1:8790/ --notes "Test build"

The version comes from the installer's name, the signature from the .sig that `npm run build` writes
beside it (only when TAURI_SIGNING_PRIVATE_KEY is set). By default the download URL points at the
release asset on mhmd2520/Hotbar, tag v<version>. An http URL only works with a test build
that allows it (README → Updates).
"""
import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path

REPO = 'https://github.com/mhmd2520/Hotbar'

ap = argparse.ArgumentParser()
ap.add_argument('setup', type=Path)
ap.add_argument('--url-base', help=f'where the installer is downloaded from (default {REPO}/releases/download/v<version>/)')
ap.add_argument('--notes', default='')
args = ap.parse_args()

m = re.fullmatch(r'Hotbar_(\d+\.\d+\.\d+)_x64-setup\.exe', args.setup.name)
if not m:
    raise SystemExit(f'not a Hotbar NSIS installer name: {args.setup.name}')
version = m[1]
sig = args.setup.with_name(args.setup.name + '.sig')
if not sig.exists():
    raise SystemExit(f'{sig} missing: build with TAURI_SIGNING_PRIVATE_KEY set')

base = args.url_base or f'{REPO}/releases/download/v{version}/'
manifest = {
    'version': version,
    'notes': args.notes,
    'pub_date': datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ'),
    'platforms': {
        'windows-x86_64': {'signature': sig.read_text().strip(), 'url': base.rstrip('/') + '/' + args.setup.name},
    },
}
out = args.setup.with_name('latest.json')
out.write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
print(out, version)
