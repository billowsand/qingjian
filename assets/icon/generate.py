"""从几何参数生成字在的 PNG 与 Windows ICO。"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[2]
OUTPUT_PNG = ROOT / "assets" / "icon" / "logo.png"
OUTPUT_ICO = ROOT / "apps" / "windows" / "tsf" / "resources" / "qingjian.ico"

CANVAS = 1024
SUPERSAMPLE = 4

COBALT_TOP = (49, 87, 216, 255)
COBALT_BOTTOM = (36, 79, 219, 255)
WARM_WHITE = (247, 246, 242, 255)
MINT = (85, 214, 194, 255)


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


if __name__ == "__main__":
    main()
