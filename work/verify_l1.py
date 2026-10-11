# -*- coding: utf-8 -*-
"""L1 精修自验：
1) VA 计数不变（L2 备份 vs 精修后）
2) 函数体零改动（每个函数：签名行→收尾 } 逐字节比对；VA 序列一致）
3) 括号配平
4) 每个函数都有注释升级
"""
import re
import sys

VA_RE = re.compile(r"^// (0x[0-9A-Fa-f]{6})")
BANNER = "// ------------------------------------------------------------"


def parse_blocks(path):
    raw = open(path, encoding="utf-8", newline="").read()
    nl = "\r\n" if "\r\n" in raw else "\n"
    lines = raw.split(nl)
    starts = [i for i, ln in enumerate(lines)
              if ln == BANNER and i + 1 < len(lines) and VA_RE.match(lines[i + 1])]
    blocks = {}
    order = []
    for bi, st in enumerate(starts):
        nxt = starts[bi + 1] if bi + 1 < len(starts) else len(lines)
        va = VA_RE.match(lines[st + 1]).group(1)
        # 签名行 = 横幅 3 行之后第一条非注释行
        k = st + 3
        while k < nxt and (lines[k].startswith("//") or lines[k].strip() == ""):
            k += 1
        sign = k
        # 函数体尾 = 本块内最后一个列 0 的 "}" 之后（跳过尾部空行/分区横幅）
        e = sign
        for j in range(sign, nxt):
            if lines[j].startswith("}"):
                e = j + 1
        body = lines[sign:e]
        blocks[va] = (st, sign, e, body, lines[st + 1])
        order.append(va)
    return blocks, order, lines


def braces(path):
    ob = cb = 0
    for ln in open(path, encoding="utf-8"):
        code = re.sub(r"//.*$", "", ln.rstrip("\r\n"))
        ob += code.count("{")
        cb += code.count("}")
    return ob, cb


def verify(l2_path, new_path, expect_va):
    b_l2, order_l2, _ = parse_blocks(l2_path)
    b_new, order_new, lines_new = parse_blocks(new_path)
    ok = True

    n_l2 = sum(1 for ln in open(l2_path, encoding="utf-8") if VA_RE.match(ln))
    n_new = sum(1 for ln in open(new_path, encoding="utf-8") if VA_RE.match(ln))
    print("  VA 计数: L2=%d 精修=%d 期望=%d -> %s" % (n_l2, n_new, expect_va,
          "OK" if n_l2 == n_new == expect_va else "FAIL"))
    ok &= n_l2 == n_new == expect_va

    print("  VA 序列一致: %s" % ("OK" if order_l2 == order_new else "FAIL"))
    ok &= order_l2 == order_new

    diff = 0
    for va in order_l2:
        _, _, _, body_l2, _ = b_l2[va]
        st_n, sign_n, e_n, body_new, _ = b_new[va]
        if body_l2 != body_new:
            diff += 1
            if diff <= 3:
                print("  BODY DIFF @%s (L2 %d 行 / 精修 %d 行)" % (
                    va, len(body_l2), len(body_new)))
                for a, b in zip(body_l2, body_new):
                    if a != b:
                        print("    L2 :", a)
                        print("    NEW:", b)
                        break
    print("  函数体逐字节比对: %d/%d 不一致 -> %s" % (diff, len(order_l2),
          "OK" if diff == 0 else "FAIL"))
    ok &= diff == 0

    ob, cb = braces(new_path)
    print("  括号配平: {%d / }%d -> %s" % (ob, cb, "OK" if ob == cb else "FAIL"))
    ok &= ob == cb

    # 注释升级覆盖：每个函数块在签名前有 【L1】 或 分区/保留注释
    no_ann = []
    for va in order_new:
        st_n, sign_n, _, _, _ = b_new[va]
        head = lines_new[st_n:sign_n]
        if not any("【L1】" in h for h in head):
            no_ann.append(va)
    print("  函数头注释覆盖: %d/%d (缺:%s)" % (
        len(order_new) - len(no_ann), len(order_new), no_ann[:5]))
    ok &= len(no_ann) == 0
    return ok


if __name__ == "__main__":
    base = r"E:\DOF LX\【40JP逆向源代码工程】"
    jobs = [
        (base + r"\work\_backup_socket_ime_L2.cpp",
         base + r"\src\platform\socket_ime.cpp", 269),
    ]
    if len(sys.argv) > 1 and sys.argv[1] == "all":
        jobs = [
            (base + r"\work\_backup_socket_ime_L2.cpp",
             base + r"\src\platform\socket_ime.cpp", 269),
            (base + r"\work\_backup_d3d8_loader_L2.cpp",
             base + r"\src\platform\d3d8_loader.cpp", 73),
            (base + r"\work\_backup_code_tail_L2.cpp",
             base + r"\src\platform\code_tail.cpp", 61),
        ]
    allok = True
    for l2, new, n in jobs:
        print(new.split("\\")[-1])
        allok &= verify(l2, new, n)
    print("ALL:", "PASS" if allok else "FAIL")
