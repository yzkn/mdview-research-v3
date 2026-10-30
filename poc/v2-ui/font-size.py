"""同梱フォント（DEC-209）の実サイズを測る。OPEN-205 の解消用。

配布 zip は PlemolJP が 211MB、IBM Plex Sans JP が 302MB あり、
実サイズを知るためだけに落とすには大きすぎる。

zip の **中央ディレクトリ（ファイル末尾）** には全エントリの非圧縮サイズが
入っているため、HTTP の Range 要求で末尾数百 KB だけ取れば各 TTF の
実サイズが分かる。ダウンロード量は 1MB 未満で済む。

実行: python font-size.py
"""

import struct
import sys
import urllib.error
import urllib.request

# 末尾から取得する量。エントリ数が多いと中央ディレクトリが伸びるため多めに取る。
TAIL_BYTES = 1024 * 1024

TARGETS = [
    (
        "PlemolJP v3.1.0",
        "https://github.com/yuru7/PlemolJP/releases/download/v3.1.0/PlemolJP_v3.1.0.zip",
        # 通常版の Regular / Bold のみ（DEC-209）。Console / 35 / イタリックは対象外。
        lambda name: (
            name.endswith(("PlemolJP-Regular.ttf", "PlemolJP-Bold.ttf"))
        ),
    ),
    (
        "IBM Plex Sans JP 3.0.0",
        "https://github.com/IBM/plex/releases/download/%40ibm%2Fplex-sans-jp%403.0.0/ibm-plex-sans-jp.zip",
        lambda name: (
            name.lower().endswith((".ttf", ".otf"))
            and ("Regular" in name or "Bold" in name)
            and "Italic" not in name
            and "SemiBold" not in name
            and "ExtraBold" not in name
            and "/ttf/" in name.replace("\\", "/")
        ),
    ),
]


def fetch(url: str, headers: dict) -> tuple[bytes, dict]:
    request = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(request, timeout=120) as response:
        return response.read(), dict(response.headers)


def total_size(url: str) -> int:
    """zip 全体のサイズを得る。HEAD が使えない場合は 0 バイトの Range で代用する。"""
    try:
        request = urllib.request.Request(url, method="HEAD")
        with urllib.request.urlopen(request, timeout=60) as response:
            length = response.headers.get("Content-Length")
            if length:
                return int(length)
    except urllib.error.HTTPError:
        pass

    # Content-Range: bytes 0-0/<total> から総サイズを読む。
    _, headers = fetch(url, {"Range": "bytes=0-0"})
    content_range = headers.get("Content-Range", "")
    if "/" in content_range:
        return int(content_range.rsplit("/", 1)[1])
    raise RuntimeError("zip の総サイズを取得できない")


def central_directory(url: str) -> list[tuple[str, int]]:
    """末尾だけ取得して、(ファイル名, 非圧縮サイズ) の一覧を返す。"""
    size = total_size(url)
    start = max(0, size - TAIL_BYTES)
    tail, headers = fetch(url, {"Range": f"bytes={start}-{size - 1}"})

    if len(tail) >= size and size > TAIL_BYTES:
        raise RuntimeError("Range 要求が効かず全体が返った。中断する")

    base = size - len(tail)
    print(f"  zip 全体 {size / 1048576:.1f}MB / 取得 {len(tail) / 1024:.0f}KB")

    # End of Central Directory を末尾から探す。
    eocd = tail.rfind(b"PK\x05\x06")
    if eocd < 0:
        raise RuntimeError("End of Central Directory が見つからない")
    cd_offset = struct.unpack("<I", tail[eocd + 16 : eocd + 20])[0]

    cursor = cd_offset - base
    if cursor < 0:
        raise RuntimeError("中央ディレクトリが取得範囲の外。TAIL_BYTES を増やす")

    entries = []
    while tail[cursor : cursor + 4] == b"PK\x01\x02":
        uncompressed = struct.unpack("<I", tail[cursor + 24 : cursor + 28])[0]
        name_len, extra_len, comment_len = struct.unpack(
            "<HHH", tail[cursor + 28 : cursor + 34]
        )
        name = tail[cursor + 46 : cursor + 46 + name_len].decode("utf-8", "replace")
        entries.append((name, uncompressed))
        cursor += 46 + name_len + extra_len + comment_len

    return entries


def main() -> int:
    grand_total = 0

    for label, url, wanted in TARGETS:
        print(f"\n## {label}")
        try:
            entries = central_directory(url)
        except Exception as error:  # noqa: BLE001 - 失敗理由をそのまま出したい
            print(f"  取得失敗: {error}")
            return 1

        print(f"  zip 内のエントリ {len(entries)} 件")
        hits = sorted((n, u) for n, u in entries if wanted(n))
        if not hits:
            print("  該当するフォントが見つからない。抽出条件を見直すこと")
            ttfs = [n for n, _ in entries if n.lower().endswith((".ttf", ".otf"))]
            for name in ttfs[:8]:
                print(f"    候補: {name}")
            continue

        subtotal = 0
        for name, uncompressed in hits:
            print(f"  {uncompressed / 1048576:6.2f} MB  {name}")
            subtotal += uncompressed
        print(f"  小計: {subtotal / 1048576:.2f} MB")
        grand_total += subtotal

    print(f"\n同梱フォント合計: {grand_total / 1048576:.2f} MB")
    return 0


if __name__ == "__main__":
    sys.exit(main())
