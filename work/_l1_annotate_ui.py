# -*- coding: utf-8 -*-
"""L1 精修应用引擎：为 src/game/ui/*.cpp 升级文件头/插入分区横幅/升级函数头注释。
铁律：只新增或修改整行注释，绝不触碰任何代码行（函数体零改动）。
安全策略：每次运行先从 work/backup_pre_refine/ui_l1_before/<file> 的干净备份出发
（备份在首次运行时从原始文件建立），保证可重复执行不叠加。
自验：VA 计数不变 / 剥离整行注释后代码行序列与备份逐行一致 / 括号配平。
用法: python work/_l1_annotate_ui.py <cfg模块名...>  （cfg 位于 work/_l1_ann_cfg/）"""
import os, re, sys, shutil, importlib, io

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
BASE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(BASE, 'src', 'game', 'ui')
BACKUP = os.path.join(BASE, 'work', 'backup_pre_refine', 'ui_l1_before')
CFGDIR = os.path.join(BASE, 'work', '_l1_ann_cfg')
sys.path.insert(0, CFGDIR)

VA_RE = re.compile(r'^// (0x[0-9A-Fa-f]{6}) ')
DASH = '// ------------------------------------------------------------'

def strip_code(lines):
    out = []
    for ln in lines:
        t = ln.strip()
        if t.startswith('//') or t == '':
            continue
        out.append(ln.rstrip('\n'))
    return out

def brace_balance(lines):
    depth = 0
    for ln in strip_code(lines):
        depth += ln.count('{') - ln.count('}')
    return depth

def apply_one(cfgmod):
    cfg = importlib.import_module(cfgmod)
    fn = cfg.FILE
    path = os.path.join(SRC, fn)
    bak = os.path.join(BACKUP, fn)
    if not os.path.exists(bak):
        os.makedirs(os.path.dirname(bak), exist_ok=True)
        shutil.copyfile(path, bak)
    with open(bak, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    orig_code = strip_code(lines)
    orig_va = sum(1 for ln in lines if VA_RE.match(ln))

    # ---- 定位 ----
    hdr_end = 0
    for i, ln in enumerate(lines):
        t = ln.strip()
        if t != '' and not t.startswith('//'):
            hdr_end = i
            break
    idx = {}
    for i, ln in enumerate(lines):
        m = VA_RE.match(ln)
        if m:
            idx[m.group(1)[2:].lower()] = i

    # ---- 构造操作表（全部基于原始行号） ----
    edit_at = {}        # 行号 -> 新文本（仅 VA 标记行）
    insert_after = {}   # 行号 -> [行]（收尾横线后插注释行；hdr 区末尾插新头）
    insert_before = {}  # 行号 -> [行]（函数横线行前插分区横幅）
    n_desc = n_extra = n_banner = 0

    for va, val in cfg.ANNOT.items():
        key = va.lower()
        if key not in idx:
            print('  !! %s: VA 0x%s 未找到' % (fn, va))
            continue
        i = idx[key]
        old = lines[i].rstrip('\n')
        desc, extra = val if isinstance(val, tuple) else (val, [])
        core = old.split(' —— ')[0]
        if desc:
            edit_at[i] = core + ' —— ' + desc + '\n'
            n_desc += 1
        if extra:
            if lines[i + 1].strip() == DASH:
                insert_after.setdefault(i + 1, []).extend(
                    (e if e.startswith('//') else '// ' + e).rstrip('\n') + '\n' for e in extra)
                n_extra += len(extra)
            else:
                print('  !! %s: VA 0x%s 后无收尾横线' % (fn, va))

    for va, banner in cfg.SECTIONS:
        key = va.lower()
        if key not in idx:
            print('  !! %s: 分区锚点 0x%s 未找到' % (fn, va))
            continue
        i = idx[key]
        opener = i - 1
        if lines[opener].strip() != DASH:
            print('  !! %s: 分区锚点 0x%s 前无横线' % (fn, va))
            continue
        insert_before.setdefault(opener, []).extend(s + '\n' for s in banner)
        n_banner += 1

    # ---- 单遍合成输出 ----
    out = []
    for i, ln in enumerate(lines):
        if i < hdr_end:
            if i == 0:
                out.extend(s + '\n' for s in cfg.HEADER)
                out.append('\n')
            continue  # 丢弃旧文件头
        if i in insert_before:
            out.extend(insert_before[i])
        out.append(edit_at.get(i, ln))
        if i in insert_after:
            out.extend(insert_after[i])
    # 旧文件头与其后代码间原有空行随 hdr_end 一起被跳过，上面已在头后补一空行

    with open(path, 'w', encoding='utf-8', newline='') as f:
        f.writelines(out)

    # ---- 自验 ----
    new_code = strip_code(out)
    ok_code = (new_code == orig_code)
    new_va = sum(1 for ln in out if VA_RE.match(ln))
    bal = brace_balance(out)
    ok = ok_code and new_va == orig_va and bal == 0
    print('%s: %s  函数头升级=%d 附加注释行=%d 分区横幅=%d  VA=%d(原%d)  代码零改动=%s  括号差=%d'
          % (fn, 'OK' if ok else 'FAIL', n_desc, n_extra, n_banner, new_va, orig_va, ok_code, bal))
    return ok

def main():
    mods = sys.argv[1:]
    if not mods:
        mods = sorted(f[12:-3] for f in os.listdir(CFGDIR) if f.startswith('_l1_ann_cfg_'))
    all_ok = True
    for m in mods:
        all_ok &= apply_one(m)
    print('ALL:', 'PASS' if all_ok else 'FAIL')

if __name__ == '__main__':
    main()
