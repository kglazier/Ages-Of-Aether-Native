"""Generate Android launcher icons from the Ages of Aether lightning bolt design."""
from PIL import Image, ImageDraw
import os, math

ANDROID_RES = r"C:\Users\kglazier\source\projects\Tower Native\android\app\src\main\res"

# Icon sizes per density
DENSITIES = {
    "mipmap-mdpi": 48,
    "mipmap-hdpi": 72,
    "mipmap-xhdpi": 96,
    "mipmap-xxhdpi": 144,
    "mipmap-xxxhdpi": 192,
}

def draw_icon(size):
    """Draw the Ages of Aether icon: gold lightning bolt on dark circle."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    cx, cy = size / 2, size / 2
    r = size * 0.47

    # Dark background circle
    draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(26, 26, 46, 255))

    # Gold ring
    ring_w = max(1, size // 20)
    r2 = r - ring_w / 2
    draw.ellipse(
        [cx - r2, cy - r2, cx + r2, cy + r2],
        outline=(218, 165, 32, 255), width=ring_w
    )

    # Lightning bolt polygon (scaled from 32x32 viewbox)
    # Original: points="18,4 12,16 17,16 14,28 22,14 17,14"
    s = size / 32.0
    bolt_points = [
        (18 * s, 4 * s),
        (12 * s, 16 * s),
        (17 * s, 16 * s),
        (14 * s, 28 * s),
        (22 * s, 14 * s),
        (17 * s, 14 * s),
    ]
    # Gold gradient approximation: use solid gold
    draw.polygon(bolt_points, fill=(255, 215, 0, 255))

    return img

for density, size in DENSITIES.items():
    d = os.path.join(ANDROID_RES, density)
    os.makedirs(d, exist_ok=True)
    icon = draw_icon(size)
    path = os.path.join(d, "ic_launcher.png")
    icon.save(path)
    print(f"  {density}: {size}x{size} -> {path}")

# Also create adaptive icon resources
anydpi = os.path.join(ANDROID_RES, "mipmap-anydpi-v26")
os.makedirs(anydpi, exist_ok=True)

# Adaptive icon XML
with open(os.path.join(anydpi, "ic_launcher.xml"), "w") as f:
    f.write('''<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@color/ic_launcher_background"/>
    <foreground android:drawable="@mipmap/ic_launcher_foreground"/>
</adaptive-icon>
''')

# Background color resource
values = os.path.join(ANDROID_RES, "values")
os.makedirs(values, exist_ok=True)
with open(os.path.join(values, "ic_launcher_background.xml"), "w") as f:
    f.write('''<?xml version="1.0" encoding="utf-8"?>
<resources>
    <color name="ic_launcher_background">#1a1a2e</color>
</resources>
''')

# Foreground (108x108 dp) at each density
FG_DENSITIES = {
    "mipmap-mdpi": 108,
    "mipmap-hdpi": 162,
    "mipmap-xhdpi": 216,
    "mipmap-xxhdpi": 324,
    "mipmap-xxxhdpi": 432,
}
for density, size in FG_DENSITIES.items():
    d = os.path.join(ANDROID_RES, density)
    os.makedirs(d, exist_ok=True)
    # Foreground: lightning bolt centered in the safe zone (66% of 108dp)
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Scale bolt to fit in the inner 66% safe zone
    inner = size * 0.66
    offset = (size - inner) / 2
    s = inner / 32.0
    bolt_points = [
        (offset + 18 * s, offset + 4 * s),
        (offset + 12 * s, offset + 16 * s),
        (offset + 17 * s, offset + 16 * s),
        (offset + 14 * s, offset + 28 * s),
        (offset + 22 * s, offset + 14 * s),
        (offset + 17 * s, offset + 14 * s),
    ]
    # Gold ring
    cx, cy = size / 2, size / 2
    r = inner * 0.47
    ring_w = max(1, int(inner) // 20)
    draw.ellipse(
        [cx - r, cy - r, cx + r, cy + r],
        outline=(218, 165, 32, 255), width=ring_w
    )
    draw.polygon(bolt_points, fill=(255, 215, 0, 255))

    path = os.path.join(d, "ic_launcher_foreground.png")
    img.save(path)
    print(f"  {density} fg: {size}x{size} -> {path}")

print("\nDone!")
