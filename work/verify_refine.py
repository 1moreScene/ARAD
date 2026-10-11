# -*- coding: utf-8 -*-
# L1 精修自验：剥离注释逐行比对 + 括号配平 + VA 计数
# 用法: python verify_refine.py <new_file> <backup_file>
import io, re, sys

def strip_comments(text):
    """移除 // 注释（字符串字面量内的 // 不动），去掉行尾空白，丢弃空行"""
    out = []
    for line in text.split("\n"):
        res = []
        i, n = 0, len(line)
        in_str = False
        while i < n:
            c = line[i]
            if in_str:
                res.append(c)
                if c == "\\" and i + 1 < n:
                    res.append(line[i+1]); i += 2; continue
                if c == '"':
                    in_str = False
                i += 1; continue
            if c == '"':
                in_str = True; res.append(c); i += 1; continue
            if c == "'" :
                # char literal（粗略：吃掉 1-2 字符）
                res.append(c); i += 1
                if i < n:
                    res.append(line[i])
                    if line[i] == "\\" and i + 1 < n:
                        res.append(line[i+1]); i += 1
                    i += 1
                continue
            if c == "/" and i + 1 < n and line[i+1] == "/":
                break  # 行注释
            res.append(c); i += 1
        stripped = "".join(res).rstrip()
        if stripped.strip():
            out.append(stripped)
    return out

def count_va(text):
    return len(re.findall(r"(?m)^// 0x[0-9A-Fa-f]{6}", text))

def strip_string_contents(code_lines):
    """去掉字符串/字符字面量内容，避免字符串内括号干扰配平统计"""
    out = []
    for line in code_lines:
        res = []
        i, n = 0, len(line)
        while i < n:
            c = line[i]
            if c == '"' or c == "'":
                q = c; i += 1
                while i < n and line[i] != q:
                    if line[i] == "\\":
                        i += 1
                    i += 1
                i += 1
                res.append(q + q)
            else:
                res.append(c); i += 1
        out.append("".join(res))
    return out

def brace_balance(code_lines):
    # 只统计代码行（已去注释、去字符串内容）中的 {}()[]
    b = p = s = 0
    for line in strip_string_contents(code_lines):
        for ch in line:
            if ch == "{": b += 1
            elif ch == "}": b -= 1
            elif ch == "(": p += 1
            elif ch == ")": p -= 1
            elif ch == "[": s += 1
            elif ch == "]": s -= 1
    return b, p, s

new_path, old_path = sys.argv[1], sys.argv[2]
new_text = io.open(new_path, encoding="utf-8").read()
old_text = io.open(old_path, encoding="utf-8").read()

va_new, va_old = count_va(new_text), count_va(old_text)
code_new = strip_comments(new_text)
code_old = strip_comments(old_text)

same = code_new == code_old
diff = 0
if not same:
    for i, (a, b) in enumerate(zip(code_old, code_new)):
        if a != b:
            print("FIRST DIFF at code line %d:\n  OLD: %s\n  NEW: %s" % (i, a, b))
            diff = i
            break
    if diff == 0 and len(code_old) != len(code_new):
        print("line count differs: old=%d new=%d" % (len(code_old), len(code_new)))

b, p, s = brace_balance(code_new)
bo, po, so = brace_balance(code_old)
print("VA count : old=%d new=%d  %s" % (va_old, va_new, "OK" if va_new == va_old else "FAIL"))
print("code line: old=%d new=%d  %s" % (len(code_old), len(code_new), "OK" if same else "FAIL"))
print("braces   : new {}=%d ()=%d []=%d / old {}=%d ()=%d []=%d  %s"
      % (b, p, s, bo, po, so, "OK" if (b, p, s) == (bo, po, so) else "FAIL"))
print("RESULT   :", "PASS" if (same and va_new == va_old and (b, p, s) == (bo, po, so)) else "FAIL")
sys.exit(0 if (same and va_new == va_old and (b, p, s) == (bo, po, so)) else 1)
