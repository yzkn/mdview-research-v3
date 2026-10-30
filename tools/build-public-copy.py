"""公開用の複製（migration-v2/）を作り直す。

**手で写さない。** 本体へ手を入れるたびに複製へ写していると、必ずどこかが
ずれる。ここから作り直せば、ずれようがない。

使い方:

    python tools/build-public-copy.py          作り直して検査する
    python tools/build-public-copy.py --check  検査だけ（作り直さない）

やること:

  1. 複製側の「生成するもの」を消す（書き下ろしの文書と .github は残す）
  2. 本体から **許したものだけ** を写す
  3. 写しながら **社名・文書名・社内向けの言い回し** を落とす
  4. 禁止語が残っていないか調べる（1 件でもあれば失敗）

**消すより、写すものを選ぶ。** 除外の一覧で管理すると、新しく足した
フォルダが黙って公開側へ入る。
"""

import getpass
import io
import os
import re
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PUBLIC = os.path.join(ROOT, "migration-v2")

# ---------------------------------------------------------------- 写すもの
#
# **ここに無いものは公開されない。** 新しいフォルダを足したら、
# 公開してよいかを考えてからここへ足す。
COPY_DIRS = [
    "src",
    # 土台（UI に依存しない部分。v3 R-01）
    "core",
    # モバイル版（Android・iOS。v3 R-02。公開側の CI が APK を組む）
    "mobile",
    "examples",
    "samples",
    "tools",
    "packaging/windows",
    "packaging/linux",
    "packaging/macos",
    # 道具の帯のアイコン（v2.5.0 R-01。自前で描いたもの）
    "assets/toolbar",
    # 依存の著作権表示の一覧（v2.6.0 R-01。実行ファイルに埋める。tools/build-licenses.py が作る）
    "assets/licenses",
]

COPY_FILES = [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "build.rs",
    "LICENSE",
    ".gitattributes",
    "assets/README.md",
    "assets/icon.svg",
    "assets/icon-small.svg",
    "assets/icon-doc.svg",
    "packaging/RELEASE-NOTES-TEMPLATE.md",
    # **`.github/` は写さない**（2026-10-13）。本体の Release（退避路）をやめ、
    # 公開側の ci.yml（組んで包む）と release.yml（載せる）は手書き（KEEP）にした
]

# 写さないもの（`COPY_DIRS` の中にあっても外す）
EXCLUDE = [
    # 社内の手順・証明書を使うもの
    # 実測値を含む
    "tools/measure",
    # **この道具そのもの。** 禁止語の一覧を持っているので、写すと必ず引っかかる
    "tools/build-public-copy.py",
]

# ---------------------------------------------------------- 複製側だけのもの
#
# **消さない。** 書き下ろした文書と、公開側の CI / Release（手書き）がここにある
KEEP = [
    "README.md",
    "CHANGELOG.md",
    ".gitignore",
    "docs",
    ".github",
    "packaging/README.md",
    "assets/fonts",
    "target",
    ".git",
]

# ---------------------------------------------------------------- 落とす語
#
# **順に当てる。** 先に長いものを当てないと、短いものが先に食う
SCRUB = [
    # 社名
    (r'authors = \["QES"\]', "authors = []"),
    # 文書名（節番号は残す）
    (r"基本設計書\s*§", "§"),
    (r"詳細設計書\s*§", "§"),
    (r"技術選定書\s*§", "§"),
    (r"実装方針書\s*§", "§"),
    (r"基本設計書", "設計メモ"),
    (r"詳細設計書", "設計メモ"),
    (r"技術選定書", "設計メモ"),
    (r"実装方針書", "設計メモ"),
    # 所属が透ける言い回し
    (r"社内文書", "業務の文書"),
    (r"社内では", "業務の文書では"),
    (r"社内", "業務"),
    # 検証用の置き場（公開側には無い）
    (r"`poc/v2-ui/font-size\.py` と同じ手口だが、あちらはサイズを読むだけ、",
     "サイズを読むだけの道具もあるが、"),
]

# ------------------------------------------------------------ 残っては困る語
FORBIDDEN = [
    "QES",
    "yuazuma",
    "Claude",
    "claude",
    "社内",
    "基本設計書",
    "詳細設計書",
    "技術選定書",
    "実装方針書",
]


def runtime_forbidden():
    """**作り直した PC のユーザー名とユーザーのフォルダ**（2026-10-13）。

    この PC で組んだ実行ファイルにはユーザー名が埋まることが分かっている（だから配布物は
    Actions で組む）。文書やソースへパスごと紛れ込むこともあるので、作り直すたびに見る。

    **CI の中では足さない。** ランナーのユーザー名は `runner` で、文書の普通の
    言葉と重なる（「runner の中で」など）。短すぎる名前も同じ理由で見ない
    """
    if os.environ.get("CI"):
        return []
    words = []
    try:
        name = getpass.getuser()
    except Exception:
        name = ""
    if len(name) >= 4:
        words.append(name)
    home = os.path.expanduser("~")
    if home and home != "~":
        words.extend({home, home.replace("\\", "/")})
    return words


# 文字として読むもの（それ以外はそのまま写す）
TEXT_SUFFIXES = (
    ".rs", ".toml", ".md", ".py", ".txt", ".yml", ".yaml",
    ".ps1", ".sh", ".iss", ".svg", ".json", ".bat",
)


RUNTIME_FORBIDDEN = runtime_forbidden()


def is_text(path):
    return path.endswith(TEXT_SUFFIXES)


def scrub(text, rel=""):
    for pattern, replacement in SCRUB:
        text = re.sub(pattern, replacement, text)
    return text


def excluded(rel):
    rel = rel.replace("\\", "/")
    return any(rel == x or rel.startswith(x + "/") for x in EXCLUDE)


def kept(rel):
    rel = rel.replace("\\", "/")
    return any(rel == x or rel.startswith(x + "/") for x in KEEP)


def clear_generated():
    """前に写したものを消す。**書き下ろしたものは残す。**"""
    removed = 0
    for name in sorted(os.listdir(PUBLIC)):
        rel = name
        if kept(rel):
            continue
        path = os.path.join(PUBLIC, name)
        if os.path.isdir(path):
            # packaging のように「一部だけ残す」ものがある
            if name in ("packaging", "assets", "tools"):
                for sub in sorted(os.listdir(path)):
                    if kept(name + "/" + sub):
                        continue
                    target = os.path.join(path, sub)
                    shutil.rmtree(target) if os.path.isdir(target) else os.remove(target)
                    removed += 1
                continue
            shutil.rmtree(path)
            removed += 1
        else:
            os.remove(path)
            removed += 1
    return removed


def copy_one(rel):
    source = os.path.join(ROOT, rel)
    target = os.path.join(PUBLIC, rel)
    os.makedirs(os.path.dirname(target), exist_ok=True)

    if is_text(source):
        try:
            # **`utf-8-sig` で読む（`utf-8` ではない）。**
            #
            # `utf-8` で読むと BOM が U+FEFF という**文字として残る。**
            # 書き出すと EF BB BF になり、下で足す BOM と合わせて
            # **2 重になる。** PowerShell は 1 つ目だけを取り除くので、
            # 2 つ目が本文の先頭に見えない文字として残り、
            # `[CmdletBinding()]` が「文の後の属性」になって構文誤りになる
            # （公開側の組み立てで実際に落ちた。2026-10-08）。
            text = io.open(source, encoding="utf-8-sig").read()
        except UnicodeDecodeError:
            shutil.copy2(source, target)
            return
        # BOM 付きで書かれているもの（.ps1 / .iss）は BOM を保つ。
        # **ここが BOM の唯一の出どころである**
        raw = open(source, "rb").read()
        bom = raw.startswith(b"\xef\xbb\xbf")
        # **改行は `.gitattributes` の決まりで書く**（2026-10-15）。`.ps1` は CRLF、それ以外は LF。
        #
        # 読むときに改行は LF へそろう（文字として読むため）。そのまま書くと、`.ps1` を
        # CRLF で取り出す CI（Linux）では作り直すたびに「変わった」と出て、
        # 「公開用の複製が最新か」が 6f75c31 から一度も通らなかった（中身は同じで改行だけ違う）。
        # 手元の作業ツリーの改行を写す作りにすると、今度は手元の CRLF が紛れ込む
        text = scrub(text, rel)
        if rel.endswith(".ps1"):
            text = text.replace("\r\n", "\n").replace("\n", "\r\n")
        body = text.encode("utf-8")
        open(target, "wb").write((b"\xef\xbb\xbf" if bom else b"") + body)
    else:
        shutil.copy2(source, target)


def copy_all():
    count = 0
    for rel in COPY_FILES:
        if excluded(rel):
            continue
        if not os.path.exists(os.path.join(ROOT, rel)):
            print("  見つからない: %s" % rel)
            continue
        copy_one(rel)
        count += 1

    for folder in COPY_DIRS:
        base = os.path.join(ROOT, folder)
        if not os.path.isdir(base):
            print("  見つからない: %s" % folder)
            continue
        for current, dirs, files in os.walk(base):
            dirs[:] = [d for d in dirs if d not in (".git", "target", "out", "__pycache__")]
            for name in files:
                rel = os.path.relpath(os.path.join(current, name), ROOT)
                if excluded(rel):
                    continue
                copy_one(rel.replace("\\", "/"))
                count += 1
    return count


def walk_public(suffixes=None):
    """複製の中のファイルを辿る。`target` と `.git` は見ない。"""
    for current, dirs, files in os.walk(PUBLIC):
        dirs[:] = [d for d in dirs if d not in (".git", "target")]
        for name in files:
            path = os.path.join(current, name)
            if suffixes and not path.endswith(suffixes):
                continue
            yield path, os.path.relpath(path, PUBLIC)


def check_newlines():
    """`.sh` が LF で BOM 無しか。

    **CRLF だと Git Bash で `$'\r': command not found` になる。**
    `bash -n` は CRLF を通すので、構文検査では気づけない。
    BOM が付いていると 1 行目の `#!` が効かない。
    """
    problems = []
    for path, rel in walk_public((".sh",)):
        raw = open(path, "rb").read()
        if raw.startswith(b"\xef\xbb\xbf"):
            problems.append("%s: BOM が付いている（.sh は BOM 禁止）" % rel)
        if b"\r\n" in raw:
            problems.append("%s: CRLF が混じっている（.sh は LF のみ）" % rel)
    return problems


def check_yaml():
    """複製側のワークフローが解析できるか。

    **本体側で通っても、写したものが壊れていないとは限らない**
    （BOM の 2 重で実際に踏んだ。2026-10-08）。
    """
    try:
        import yaml
    except ImportError:
        return ["（飛ばした）PyYAML が無いため YAML を解析していない"]

    problems = []
    for path, rel in walk_public((".yml", ".yaml")):
        try:
            with io.open(path, encoding="utf-8-sig") as handle:
                yaml.safe_load(handle.read())
        except Exception as error:
            problems.append("%s: YAML として読めない（%s）" % (rel, error))
    return problems


def check_actions_drift():
    """本体と公開側のワークフローで、**同じ Action の版が違う**ものを知らせる（2026-10-13）。

    公開側の Dependabot はやめた（依存の更新は本体から写る）。ただし公開側の
    ワークフローは手書きで写さないので、**本体の Dependabot が Actions を上げても
    公開側は古いまま**になる。古い版はいつか使えなくなる（Node の版の打ち切りなど）。

    **止めずに知らせる。** 本体の Dependabot が上げた直後は、公開側を直すまで
    必ず食い違うため。知らせが出たら、公開側の 2 つに同じ版を入れる
    """
    pattern = re.compile(r"uses:\s*([\w.-]+/[\w./-]+)@(\S+)")

    def versions(paths):
        found = {}
        for path in paths:
            if not os.path.exists(path):
                continue
            for name, version in pattern.findall(io.open(path, encoding="utf-8").read()):
                found.setdefault(name, set()).add(version)
        return found

    body_dir = os.path.join(ROOT, ".github", "workflows")
    public_dir = os.path.join(PUBLIC, ".github", "workflows")
    body = versions([os.path.join(body_dir, n) for n in sorted(os.listdir(body_dir))] if os.path.isdir(body_dir) else [])
    public = versions([os.path.join(public_dir, n) for n in sorted(os.listdir(public_dir))] if os.path.isdir(public_dir) else [])
    notes = []
    for name in sorted(set(body) & set(public)):
        if body[name] != public[name]:
            notes.append("（知らせ）%s の版が違う: 本体 %s / 公開側 %s。公開側の ci.yml・release.yml を本体に揃える"
                         % (name, ",".join(sorted(body[name])), ",".join(sorted(public[name]))))
    return notes


def check_powershell():
    """複製側の `.ps1` が構文として通るか。

    **Windows でしか見られない。** 見ていないときは、そう出す。
    """
    if os.name != "nt":
        return ["（飛ばした）Windows ではないため .ps1 を構文検査していない"]
    if not shutil.which("pwsh"):
        return ["（飛ばした）pwsh が無いため .ps1 を構文検査していない"]

    scripts = [path for path, _ in walk_public((".ps1",))]
    if not scripts:
        return []

    # **1 回の起動でまとめて見る。** 1 本ずつ起動すると遅い。
    #
    # **パスは文に埋め込む。** `pwsh -Command` は後ろに並べた引数を
    # `$args` に束縛しないため、渡したつもりで 1 本も見ていない状態に
    # なる（実際に踏んだ。壊して確かめて気づいた）
    listed = ", ".join("'" + p.replace("'", "''") + "'" for p in scripts)
    script = (
        "$bad = 0; "
        "foreach ($p in @(" + listed + ")) { "
        "  $errs = $null; "
        "  [void][System.Management.Automation.Language.Parser]::ParseFile("
        "    $p, [ref]$null, [ref]$errs); "
        "  if ($errs) { $bad++; Write-Output \"$p :: $($errs[0].Message)\" } "
        "}; "
        "exit $bad"
    )
    done = subprocess.run(
        ["pwsh", "-NoProfile", "-Command", script],
        capture_output=True,
        text=True,
    )
    problems = []
    for line in done.stdout.splitlines():
        if "::" in line:
            path, _, message = line.partition("::")
            rel = os.path.relpath(path.strip(), PUBLIC)
            problems.append("%s: 構文誤り（%s）" % (rel, message.strip()))
    return problems


def check():
    """禁止語・BOM・改行を調べる（1 度だけ辿る）。"""
    hits = []
    bom = []
    for current, dirs, files in os.walk(PUBLIC):
        dirs[:] = [d for d in dirs if d not in (".git", "target")]
        for name in files:
            path = os.path.join(current, name)
            if not is_text(path):
                continue
            try:
                text = io.open(path, encoding="utf-8-sig").read()
            except UnicodeDecodeError:
                continue
            rel = os.path.relpath(path, PUBLIC)

            # **本文の中に BOM が残っていないか。**
            #
            # `utf-8-sig` は先頭の 1 つだけを剥がす。2 つ目以降は
            # U+FEFF の文字として残り、**見えないまま構文を壊す。**
            # 禁止語の検査では引っかからないので、別に見る
            if "\ufeff" in text:
                at = next(
                    (n for n, l in enumerate(text.splitlines(), 1) if "\ufeff" in l),
                    0,
                )
                bom.append("%s:%d に BOM が残っている（2 重の BOM）" % (rel, at))

            for word in FORBIDDEN + RUNTIME_FORBIDDEN:
                if word in text:
                    line = next(
                        (n for n, l in enumerate(text.splitlines(), 1) if word in l),
                        0,
                    )
                    hits.append((rel, word, line))
    return hits, bom


def main():
    only_check = "--check" in sys.argv

    if not only_check:
        print("作り直す: %s" % PUBLIC)
        removed = clear_generated()
        print("  消した: %d" % removed)
        copied = copy_all()
        print("  写した: %d" % copied)

    hits, bom = check()

    # **実行せずに見られるものを、まとめて見る。**
    #
    # `cargo test` は Rust のソースしか触らない。配布物を組む
    # スクリプトは一度も通らないので、別に見ないと気づけない
    # （BOM の 2 重で実際に踏んだ。2026-10-08）
    broken = bom + check_newlines() + check_yaml() + check_powershell()
    skipped = [x for x in broken if x.startswith("（飛ばした）")]
    broken = [x for x in broken if not x.startswith("（飛ばした）")]

    print()
    if broken:
        print("**壊れている: %d 件**" % len(broken))
        for line in broken:
            print("  " + line)
        return 1

    if hits:
        print("**禁止語が残っている: %d 件**" % len(hits))
        for rel, word, line in hits[:30]:
            print("  %s:%d  %s" % (rel, line, word))
        return 1

    print("禁止語は残っていない（%d 語を検査）" % len(FORBIDDEN))
    print("BOM の 2 重・.sh の改行・YAML・.ps1 の構文も見た")

    # **飛ばしたものは黙らない。** 確かめていないことを
    # 確かめたことにしない
    for line in skipped:
        print("  " + line)
    for line in check_actions_drift():
        print("  " + line)
    return 0


if __name__ == "__main__":
    sys.exit(main())
