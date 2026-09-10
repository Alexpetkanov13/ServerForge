"""Build branded Windows icons and NSIS bitmaps from generated artwork."""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
ASSETS = Path(r"C:\Users\User\.cursor\projects\c-React-Projects-ServerForge\assets")
ICONS = ROOT / "src-tauri" / "icons"
WINDOWS = ROOT / "src-tauri" / "windows" / "branding"

ZINC = (18, 18, 20, 255)
TEAL = (45, 212, 191, 255)
WHITE = (250, 250, 250, 255)
MUTED = (161, 161, 170, 255)


def load_or_none(name: str) -> Image.Image | None:
    path = ASSETS / name
    if path.exists():
        return Image.open(path).convert("RGBA")
    return None


def cover(im: Image.Image, size: tuple[int, int]) -> Image.Image:
    tw, th = size
    scale = max(tw / im.width, th / im.height)
    nw, nh = max(1, int(im.width * scale)), max(1, int(im.height * scale))
    resized = im.resize((nw, nh), Image.Resampling.LANCZOS)
    left = (nw - tw) // 2
    top = (nh - th) // 2
    return resized.crop((left, top, left + tw, top + th))


def contain_on_canvas(im: Image.Image, size: tuple[int, int], bg: tuple[int, int, int, int]) -> Image.Image:
    canvas = Image.new("RGBA", size, bg)
    scale = min(size[0] / im.width, size[1] / im.height)
    nw, nh = max(1, int(im.width * scale)), max(1, int(im.height * scale))
    resized = im.resize((nw, nh), Image.Resampling.LANCZOS)
    canvas.paste(resized, ((size[0] - nw) // 2, (size[1] - nh) // 2), resized)
    return canvas


def fallback_icon(size: int = 1024) -> Image.Image:
    im = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    pad = int(size * 0.08)
    d.rounded_rectangle([pad, pad, size - pad, size - pad], radius=int(size * 0.22), fill=ZINC)
    cx, cy = size // 2, int(size * 0.52)
    w = int(size * 0.42)
    h = int(size * 0.38)
    d.polygon(
        [(cx, cy - h), (cx + w, cy + int(h * 0.35)), (cx - w, cy + int(h * 0.35))],
        fill=TEAL,
    )
    bw, bh = int(size * 0.28), int(size * 0.12)
    d.rounded_rectangle(
        [cx - bw, cy + int(h * 0.28), cx + bw, cy + int(h * 0.28) + bh],
        radius=int(size * 0.03),
        fill=TEAL,
    )
    return im


def fallback_sidebar() -> Image.Image:
    w, h = 164 * 4, 314 * 4
    im = Image.new("RGBA", (w, h), ZINC)
    d = ImageDraw.Draw(im)
    for y in range(h):
        mix = y / h
        color = (
            int(12 + 20 * mix),
            int(12 + 40 * mix),
            int(14 + 36 * mix),
            255,
        )
        d.line([(0, y), (w, y)], fill=color)
    mark = fallback_icon(420)
    im.alpha_composite(mark, ((w - mark.width) // 2, 80))
    try:
        font = ImageFont.truetype("segoeui.ttf", 48)
        small = ImageFont.truetype("segoeui.ttf", 22)
    except OSError:
        font = ImageFont.load_default()
        small = font
    d.text((w // 2, 560), "SERVERFORGE", fill=WHITE, font=font, anchor="mm")
    d.text((w // 2, 620), "Minecraft Server Control", fill=MUTED, font=small, anchor="mm")
    return im


def fallback_header() -> Image.Image:
    w, h = 150 * 4, 57 * 4
    im = Image.new("RGBA", (w, h), ZINC)
    d = ImageDraw.Draw(im)
    mark = fallback_icon(h - 24)
    im.alpha_composite(mark, (16, 12))
    try:
        font = ImageFont.truetype("segoeui.ttf", 42)
    except OSError:
        font = ImageFont.load_default()
    d.text((h + 8, h // 2), "SERVERFORGE", fill=WHITE, font=font, anchor="lm")
    d.line([(h + 8, h - 28), (w - 24, h - 28)], fill=TEAL, width=3)
    return im


def to_bmp(im: Image.Image, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    im.convert("RGB").save(dest, format="BMP")


def main() -> None:
    ICONS.mkdir(parents=True, exist_ok=True)
    WINDOWS.mkdir(parents=True, exist_ok=True)

    icon = load_or_none("serverforge-logo.png") or load_or_none("serverforge-icon.png") or fallback_icon()
    sidebar = load_or_none("serverforge-installer-sidebar.png") or fallback_sidebar()
    header = load_or_none("serverforge-installer-header.png") or fallback_header()

    for size, name in [(32, "32x32.png"), (128, "128x128.png"), (256, "128x128@2x.png")]:
        contain_on_canvas(icon, (size, size), (0, 0, 0, 0)).save(ICONS / name, format="PNG")

    ico_sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    icon.save(ICONS / "icon.ico", format="ICO", sizes=ico_sizes)

    to_bmp(cover(sidebar, (164, 314)), WINDOWS / "sidebar.bmp")
    to_bmp(cover(header, (150, 57)), WINDOWS / "header.bmp")
    contain_on_canvas(icon, (256, 256), (0, 0, 0, 0)).save(WINDOWS / "installer-icon.png")
    (WINDOWS / "installer.ico").write_bytes((ICONS / "icon.ico").read_bytes())

    print("Wrote icons to", ICONS)
    print("Wrote installer bitmaps to", WINDOWS)


if __name__ == "__main__":
    main()
