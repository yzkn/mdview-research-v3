"""PDF を絵にして、絵文字の行ごとに色の画素を数える（v3 の M-7 の検証）。

python check.py <PDF> <出力の PNG>
行ごとの色の画素が 0 なら、その行の絵文字は色で描けていない（豆腐・字の形・何も無い）。
"""
import sys

import fitz  # PyMuPDF
from PIL import Image

LINES = ["1 basic", "2 heart (FE0F)", "3 warning (FE0F)", "4 ZWJ family", "5 flag", "6 skin tone"]

pdf, png = sys.argv[1], sys.argv[2]
page = fitz.open(pdf)[0]
scale = 2.0
page.get_pixmap(matrix=fitz.Matrix(scale, scale)).save(png)
img = Image.open(png).convert("RGB")
rows = []
for i, label in enumerate(LINES):
    # main.rs と同じ並び: 基準線 y = 60 + 56 i、字の大きさ 36
    top = int((60 + 56 * i - 40) * scale)
    bottom = int((60 + 56 * i + 12) * scale)
    band = img.crop((0, max(0, top), img.width, min(img.height, bottom)))
    colored = sum(1 for r, g, b in band.getdata() if max(r, g, b) - min(r, g, b) > 60)
    rows.append((label, colored))
for label, colored in rows:
    print(f"|{label}|{colored}|{'色で出た' if colored > 200 else '**色が無い**'}|")
