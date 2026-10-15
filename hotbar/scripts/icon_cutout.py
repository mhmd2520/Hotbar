r"""Cut the keycap out of the dark rounded-square icon, so the app icon has a transparent background.

    python scripts\icon_cutout.py            # branding\hotbar-icon-dark-bg.png -> branding\hotbar-icon.png

Keycap = everything clearly brighter than the near-black backdrop (holes such as the orange slash are
filled). Its edge is anti-aliased and un-mixed from the backdrop colour, the old baked-in shadow is
dropped, and a soft transparent shadow is drawn instead. The key is then scaled to fill the icon.
"""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

BRANDING = Path(__file__).resolve().parents[2] / "branding"
SRC = BRANDING / "hotbar-icon-dark-bg.png"
OUT = BRANDING / "hotbar-icon.png"
SIZE = 1024
FILL = 0.90  # keycap width as a share of the icon


def main() -> None:
    im = Image.open(SRC).convert("RGB")
    w, h = im.size
    bg = im.getpixel((60, h // 2))  # the backdrop colour, left of the key
    lum = im.convert("L")

    # 1. solid keycap mask: bright pixels, holes filled (flood the outside from a corner, invert)
    solid = lum.point(lambda v: 255 if v > 55 else 0)
    outside = solid.copy()
    ImageDraw.floodfill(outside, (0, 0), 128)
    mask = outside.point(lambda v: 0 if v == 128 else 255)

    # 2. soft edge: shrink a little, feather, so the rim is anti-aliased and loses the dark fringe
    mask = mask.filter(ImageFilter.MinFilter(3)).filter(ImageFilter.GaussianBlur(1.1))

    # 3. un-mix the backdrop from semi-transparent rim pixels: c = (p - (1 - a) * bg) / a
    px, mpx = im.load(), mask.load()
    for y in range(h):
        for x in range(w):
            a = mpx[x, y] / 255
            if 0 < a < 1:
                px[x, y] = tuple(max(0, min(255, round((c - (1 - a) * b) / a))) for c, b in zip(px[x, y], bg))
    key = im.convert("RGBA")
    key.putalpha(mask)

    # 4. crop to the key, scale it to fill the icon
    key = key.crop(mask.getbbox())
    scale = SIZE * FILL / key.width
    key = key.resize((round(key.width * scale), round(key.height * scale)), Image.Resampling.LANCZOS)

    # 5. soft shadow from the key's own shape
    canvas = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    x0, y0 = (SIZE - key.width) // 2, (SIZE - key.height) // 2 - 10
    shadow_alpha = Image.new("L", (SIZE, SIZE), 0)
    shadow_alpha.paste(key.getchannel("A").point(lambda v: v * 0.42), (x0, y0 + 22))
    shadow_alpha = shadow_alpha.filter(ImageFilter.GaussianBlur(18))
    shadow = Image.new("RGBA", (SIZE, SIZE), (20, 12, 8, 0))
    shadow.putalpha(shadow_alpha)
    canvas = Image.alpha_composite(canvas, shadow)
    canvas.alpha_composite(key, (x0, y0))

    # sanity: no opaque backdrop left in the corners or along the edges
    a = canvas.getchannel("A")
    assert max(a.getpixel((x, y)) for x, y in [(5, 5), (SIZE - 5, 5), (5, SIZE - 5), (SIZE - 5, SIZE - 5)]) == 0
    canvas.save(OUT, optimize=True)
    print(f"{OUT.name}: key {key.width}x{key.height} at ({x0},{y0}), backdrop {bg}")


if __name__ == "__main__":
    main()
