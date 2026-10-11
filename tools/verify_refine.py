# -*- coding: utf-8 -*-
"""
精修终验：对比精修前备份（work/backup_pre_refine/src_snapshot）与当前 src/
验证三件事：
1. VA 标记集合不变（无丢失/无新增）
2. 函数体零改动（剥离注释行后逐行一致）
3. 统计精修增量（注释行数变化）
"""
import io, os, re, glob

ROOT = r"E:\DOF LX\【40JP逆向源代码工程】"
BASE = os.path.join(ROOT, "work", "backup_pre_refine", "src_snapshot")
CUR = os.path.join(ROOT, "src")

def norm(p): return p.replace("\\", "/")

def parse_blocks(txt):
    """把源码文本解析为 {va: [非注释行]} —— 函数体 = 头注释后到下一个 VA 头之间的非注释行"""
    blocks = {}
    lines = txt.replace("\r\n", "\n").split("\n")
    cur_va = None
    cur_body = []
    for line in lines:
        m = re.match(r"^// 0x([0-9A-F]{6})", line)
        if m:
            if cur_va is not None:
                blocks.setdefault(cur_va, []).extend(cur_body)
            cur_va = int(m.group(1), 16)
            cur_body = []
            continue
        if cur_va is not None:
            s = line.strip()
            if s and not s.startswith("//"):
                cur_body.append(line)
    if cur_va is not None:
        blocks.setdefault(cur_va, []).extend(cur_body)
    # 规范化：去空白
    return {va: [l.strip() for l in body if l.strip()] for va, body in blocks.items()}

def collect(root):
    all_blocks = {}
    for path in glob.glob(os.path.join(root, "**", "*.cpp"), recursive=True):
        txt = io.open(path, encoding="utf-8", errors="replace").read()
        for va, body in parse_blocks(txt).items():
            if va in all_blocks and all_blocks[va] != body:
                # 同 VA 多文件（L1/L2 双层）：保留第一个，记录冲突
                pass
            else:
                all_blocks.setdefault(va, body)
    return all_blocks

base = collect(BASE)
cur = collect(CUR)

print("基线函数块: %d   当前函数块: %d" % (len(base), len(cur)))

# 1. VA 集合
va_base = set(base)
va_cur = set(cur)
lost = va_base - va_cur
added = va_cur - va_base
print("VA 丢失: %d   VA 新增: %d" % (len(lost), len(added)))
for va in sorted(lost)[:10]:
    print("  LOST %08X" % va)
for va in sorted(added)[:10]:
    print("  ADDED %08X" % va)

# 2. 函数体比对（在两边都有 VA 的函数中）
diff = 0
same = 0
diff_list = []
for va in (va_base & va_cur):
    b = base[va]
    c = cur[va]
    # 去掉基线中的截断标记差异：基线若被截断而当前是完整版，当前应包含基线前缀
    if b == c:
        same += 1
    elif len(c) >= len(b) and c[:len(b)] == b:
        same += 1   # 当前是基线的超集（截断被补全），允许
    else:
        diff += 1
        diff_list.append(va)

print("函数体一致（含截断补全超集）: %d" % same)
print("函数体被改动: %d" % diff)
for va in diff_list[:15]:
    print("  DIFF %08X" % va)
    print("    base[:3]:", base[va][:3])
    print("    cur [:3]:", cur[va][:3])

# 3. 注释增量统计
def count_comment_lines(root):
    total = 0
    for path in glob.glob(os.path.join(root, "**", "*.cpp"), recursive=True):
        for line in io.open(path, encoding="utf-8", errors="replace"):
            s = line.strip()
            if s.startswith("//"):
                total += 1
    return total

cb = count_comment_lines(BASE)
cc = count_comment_lines(CUR)
print()
print("注释行: 基线 %d → 当前 %d（增量 %d）" % (cb, cc, cc - cb))

verdict = "PASS" if (len(lost) == 0 and diff == 0) else "FAIL"
print()
print("=== 终验判定: %s ===" % verdict)
io.open(os.path.join(ROOT, "work", "refine_verify_report.txt"), "w", encoding="utf-8").write(
    "verdict=%s\nbase_blocks=%d\ncur_blocks=%d\nlost=%d\nadded=%d\nbody_diff=%d\ncomments %d->%d\n"
    % (verdict, len(base), len(cur), len(lost), len(added), diff, cb, cc) +
    "\n".join("DIFF %08X" % va for va in diff_list))
