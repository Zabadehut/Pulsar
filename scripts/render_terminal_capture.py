#!/usr/bin/env python3
import argparse
import html
import os
import re


CSI_RE = re.compile(rb"\x1b\[([0-9;?]*)([A-Za-z])")


class Screen:
    def __init__(self, rows: int, cols: int) -> None:
        self.rows = rows
        self.cols = cols
        self.grid = [[" "] * cols for _ in range(rows)]
        self.row = 0
        self.col = 0
        self.alt_active = False

    def clear(self) -> None:
        self.grid = [[" "] * self.cols for _ in range(self.rows)]
        self.row = 0
        self.col = 0

    def move(self, row: int, col: int) -> None:
        self.row = max(0, min(self.rows - 1, row))
        self.col = max(0, min(self.cols - 1, col))

    def put(self, ch: str) -> None:
        if ch == "\n":
            self.row = min(self.rows - 1, self.row + 1)
            self.col = 0
            return
        if ch == "\r":
            self.col = 0
            return
        if ch == "\b":
            self.col = max(0, self.col - 1)
            return
        if ord(ch) < 32:
            return
        if 0 <= self.row < self.rows and 0 <= self.col < self.cols:
            self.grid[self.row][self.col] = ch
        self.col += 1
        if self.col >= self.cols:
            self.col = self.cols - 1


def feed(screen: Screen, data: bytes) -> None:
    i = 0
    while i < len(data):
        byte = data[i]
        if byte == 0x1B:
            if i + 1 < len(data) and data[i + 1] == ord("["):
                match = CSI_RE.match(data, i)
                if not match:
                    i += 1
                    continue
                params = match.group(1).decode("ascii", errors="ignore")
                code = match.group(2).decode("ascii", errors="ignore")
                values = [
                    int(part) if part and part.isdigit() else 0
                    for part in params.replace("?", "").split(";")
                ]
                if params.startswith("?1049") and code == "h":
                    screen.alt_active = True
                    screen.clear()
                    i = match.end()
                    continue
                if params.startswith("?1049") and code == "l":
                    i = match.end()
                    continue
                if code == "H":
                    row = (values[0] if len(values) >= 1 and values[0] else 1) - 1
                    col = (values[1] if len(values) >= 2 and values[1] else 1) - 1
                    screen.move(row, col)
                elif code == "J":
                    if not values or values[0] in (0, 2):
                        screen.clear()
                elif code == "K":
                    for col in range(screen.col, screen.cols):
                        screen.grid[screen.row][col] = " "
                elif code in ("A", "B", "C", "D"):
                    amount = values[0] if values and values[0] else 1
                    if code == "A":
                        screen.move(screen.row - amount, screen.col)
                    elif code == "B":
                        screen.move(screen.row + amount, screen.col)
                    elif code == "C":
                        screen.move(screen.row, screen.col + amount)
                    elif code == "D":
                        screen.move(screen.row, screen.col - amount)
                i = match.end()
                continue
            i += 1
            continue

        if not screen.alt_active:
            i += 1
            continue

        width = utf8_char_width(byte)
        try:
            ch = data[i : i + width].decode("utf-8")
        except UnicodeDecodeError:
            ch = "�"
            width = 1
        screen.put(ch)
        i += width


def utf8_char_width(first_byte: int) -> int:
    if first_byte < 0x80:
        return 1
    if (first_byte & 0xE0) == 0xC0:
        return 2
    if (first_byte & 0xF0) == 0xE0:
        return 3
    if (first_byte & 0xF8) == 0xF0:
        return 4
    return 1


def render_svg(screen: Screen, title: str) -> str:
    font_size = 14
    line_height = 18
    left = 20
    top = 26
    width = left * 2 + screen.cols * 8
    height = top * 2 + screen.rows * line_height + 18
    lines = ["".join(row).rstrip() for row in screen.grid]

    svg = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" aria-label="{html.escape(title)}">',
        "<defs>",
        '<linearGradient id="bg" x1="0" x2="1" y1="0" y2="1">',
        '<stop offset="0%" stop-color="#0f172a"/>',
        '<stop offset="100%" stop-color="#111827"/>',
        "</linearGradient>",
        "</defs>",
        f'<rect width="{width}" height="{height}" fill="url(#bg)"/>',
        f'<rect x="10" y="10" width="{width - 20}" height="{height - 20}" rx="12" fill="#0b1020" stroke="#334155" stroke-width="1.5"/>',
    ]
    for index, line in enumerate(lines):
        text = html.escape(line if line else " ")
        y = top + index * line_height
        svg.append(
            f'<text x="{left}" y="{y}" fill="#e5e7eb" font-size="{font_size}" font-family="ui-monospace, SFMono-Regular, Menlo, Consolas, monospace" xml:space="preserve">{text}</text>'
        )
    svg.append("</svg>")
    return "\n".join(svg)


def main() -> int:
    parser = argparse.ArgumentParser(description="Render a terminal typescript capture into an SVG.")
    parser.add_argument("--input", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--rows", type=int, default=40)
    parser.add_argument("--cols", type=int, default=140)
    parser.add_argument("--title", default="Sysray terminal capture")
    args = parser.parse_args()

    with open(args.input, "rb") as handle:
        data = handle.read()

    screen = Screen(args.rows, args.cols)
    feed(screen, data)
    svg = render_svg(screen, args.title)

    os.makedirs(os.path.dirname(os.path.abspath(args.output)), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as handle:
        handle.write(svg)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
