"""从几何参数生成字在的 PNG 与 Windows ICO。"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[2]
OUTPUT_PNG = ROOT / "assets" / "icon" / "logo.png"
OUTPUT_ICO = ROOT / "apps" / "windows" / "tsf" / "resources" / "qingjian.ico"
OUTPUT_WIZARD_LIGHT = ROOT / "assets" / "icon" / "installer-wizard-light.png"
OUTPUT_WIZARD_DARK = ROOT / "assets" / "icon" / "installer-wizard-dark.png"

CANVAS = 1024
SUPERSAMPLE = 4

COBALT_TOP = (49, 87, 216, 255)
COBALT_BOTTOM = (36, 79, 219, 255)
WARM_WHITE = (247, 246, 242, 255)
MINT = (85, 214, 194, 255)
DARK_NAVY = (23, 32, 51, 255)
LIGHT_HIGHLIGHT = (232, 238, 255, 255)
DARK_HIGHLIGHT = (37, 55, 95, 255)


def scaled(value: float) -> int:
    return round(value * SUPERSAMPLE)


def point(value: tuple[float, float]) -> tuple[int, int]:
    return scaled(value[0]), scaled(value[1])


def cubic_points(
    start: tuple[float, float],
    control_a: tuple[float, float],
    control_b: tuple[float, float],
    end: tuple[float, float],
    count: int = 96,
) -> list[tuple[int, int]]:
    points = []
    for index in range(count + 1):
        t = index / count
        u = 1.0 - t
        x = (
            u**3 * start[0]
            + 3 * u**2 * t * control_a[0]
            + 3 * u * t**2 * control_b[0]
            + t**3 * end[0]
        )
        y = (
            u**3 * start[1]
            + 3 * u**2 * t * control_a[1]
            + 3 * u * t**2 * control_b[1]
            + t**3 * end[1]
        )
        points.append(point((x, y)))
    return points


def rounded_line(
    draw: ImageDraw.ImageDraw,
    points: list[tuple[int, int]],
    width: float,
    color: tuple[int, int, int, int],
) -> None:
    px = scaled(width)
    radius = px // 2
    draw.line(points, fill=color, width=px, joint="curve")
    for x, y in (points[0], points[-1]):
        draw.ellipse((x - radius, y - radius, x + radius, y + radius), fill=color)


def background() -> Image.Image:
    size = CANVAS * SUPERSAMPLE
    gradient = Image.new("RGBA", (1, size))
    pixels = gradient.load()
    for y in range(size):
        t = y / max(size - 1, 1)
        pixels[0, y] = tuple(
            round(COBALT_TOP[channel] * (1 - t) + COBALT_BOTTOM[channel] * t)
            for channel in range(4)
        )
    gradient = gradient.resize((size, size))
    mask = Image.new("L", (size, size))
    ImageDraw.Draw(mask).rounded_rectangle(
        (scaled(64), scaled(64), scaled(960), scaled(960)),
        radius=scaled(208),
        fill=255,
    )
    canvas = Image.new("RGBA", (size, size))
    canvas.paste(gradient, mask=mask)
    return canvas


def render() -> Image.Image:
    image = background()
    draw = ImageDraw.Draw(image)

    dot = point((300, 255))
    dot_radius = scaled(44)
    draw.ellipse(
        (
            dot[0] - dot_radius,
            dot[1] - dot_radius,
            dot[0] + dot_radius,
            dot[1] + dot_radius,
        ),
        fill=MINT,
    )

    rounded_line(
        draw,
        cubic_points((210, 390), (380, 315), (650, 315), (800, 390)),
        72,
        WARM_WHITE,
    )

    lower_left = [
        point((210, 500)),
        point((210, 680)),
        *cubic_points((210, 680), (210, 735), (235, 760), (290, 760))[1:],
        point((600, 760)),
    ]
    rounded_line(draw, lower_left, 72, WARM_WHITE)
    rounded_line(
        draw,
        cubic_points((720, 760), (775, 760), (810, 720), (810, 650)),
        72,
        WARM_WHITE,
    )
    rounded_line(draw, [point((500, 470)), point((500, 650))], 50, MINT)

    return image.resize((CANVAS, CANVAS), Image.Resampling.LANCZOS)


def render_wizard(icon: Image.Image, *, dark: bool) -> Image.Image:
    """生成 Inno Setup 欢迎页 / 完成页左侧的 164:314 品牌画面。"""

    width, height = 492, 942
    background = DARK_NAVY if dark else WARM_WHITE
    quiet = DARK_HIGHLIGHT if dark else LIGHT_HIGHLIGHT
    ink = WARM_WHITE if dark else DARK_NAVY
    image = Image.new("RGBA", (width, height), background)
    draw = ImageDraw.Draw(image)

    # 从画面边缘探进来的开放圆角框呼应候选窗，不做成封闭的徽章或盾牌。
    draw.rounded_rectangle((-112, 92, 360, 564), radius=120, fill=quiet)
    draw.rounded_rectangle(
        (198, 650, 590, 1042),
        radius=104,
        outline=COBALT_BOTTOM,
        width=18,
    )

    mark = icon.resize((270, 270), Image.Resampling.LANCZOS)
    image.alpha_composite(mark, ((width - mark.width) // 2, 180))

    # 下半部是一条抽象输入栏：钴蓝表示当前项，薄荷表示输入光标。
    bar_left, bar_top, bar_right, bar_bottom = 74, 610, 418, 724
    draw.rounded_rectangle(
        (bar_left, bar_top, bar_right, bar_bottom),
        radius=32,
        fill=background,
        outline=COBALT_TOP if not dark else (107, 139, 255, 255),
        width=12,
    )
    draw.rounded_rectangle(
        (bar_left + 28, bar_top + 26, bar_left + 160, bar_bottom - 26),
        radius=20,
        fill=quiet,
    )
    draw.rounded_rectangle(
        (bar_left + 184, bar_top + 22, bar_left + 194, bar_bottom - 22),
        radius=5,
        fill=MINT,
    )
    draw.rounded_rectangle(
        (bar_left + 220, bar_top + 42, bar_right - 28, bar_top + 54),
        radius=6,
        fill=ink,
    )

    draw.ellipse((82, 804, 118, 840), fill=MINT)
    draw.rounded_rectangle((140, 814, 360, 830), radius=8, fill=ink)
    return image.convert("RGB")


def main() -> None:
    icon = render()
    OUTPUT_PNG.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_ICO.parent.mkdir(parents=True, exist_ok=True)
    icon.save(OUTPUT_PNG, optimize=True)
    icon.save(
        OUTPUT_ICO,
        format="ICO",
        sizes=[(16, 16), (20, 20), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )
    render_wizard(icon, dark=False).save(OUTPUT_WIZARD_LIGHT, optimize=True)
    render_wizard(icon, dark=True).save(OUTPUT_WIZARD_DARK, optimize=True)


if __name__ == "__main__":
    main()
