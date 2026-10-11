# -*- coding: utf-8 -*-
"""L1 精修工具：纯插入式注释升级（函数体零改动的可证明实现）。

原理：
  new_file = 新文件头 + 旧文件其余行 + 若干"插入行"（全部为 // 注释或空行）
  自验时从 new_file 剔除全部插入行，必须逐字节还原出旧文件（剔除头块替换）。
"""
import re
import sys

BANNER = "// ------------------------------------------------------------"
VA_RE = re.compile(r"^// 0x[0-9A-Fa-f]{6}")


def parse_sections(text):
    """解析文本块: '@0xVA' 行开段，后续非空行归入该段。"""
    sec, cur = {}, None
    for ln in text.split("\n"):
        if ln.startswith("@0x"):
            cur = ln[1:].strip()
            sec[cur] = []
        elif cur and ln.strip():
            sec[cur].append(ln)
    return sec


def parse_annots(text):
    """解析文本块: '0xVA | 首行' 开注，后续非空行为续行（原样剥空格）。"""
    ann, cur = {}, None
    for ln in text.split("\n"):
        if not ln.strip():
            continue
        if ln.startswith("0x"):
            va, _, first = ln.partition("|")
            cur = va.strip()
            ann[cur] = ["// " + first.strip()]
        else:
            ann[cur].append("// " + ln.strip())
    return ann


class Refiner:
    def __init__(self, path):
        self.path = path
        raw = open(path, encoding="utf-8", newline="").read()
        assert "\r\n" in raw, path
        self.old_lines = raw.split("\r\n")
        self.new_pairs = []          # list of (line_text, inserted:bool)

    # ---- 定位函数块 ----
    def _banner_starts(self):
        return [
            i for i, ln in enumerate(self.old_lines)
            if ln == BANNER and i + 1 < len(self.old_lines) and VA_RE.match(self.old_lines[i + 1])
        ]

    def va_of_block(self, banner_idx):
        m = re.match(r"^// (0x[0-9A-Fa-f]{6})", self.old_lines[banner_idx + 1])
        return m.group(1)

    # ---- 头块替换 ----
    def set_header(self, header_lines):
        starts = self._banner_starts()
        first_banner = starts[0]
        head = self.old_lines[:first_banner]
        # 头块 = 顶部连续的注释行（含 ==== 框与 #include 之前的全部注释）
        j = 0
        while j < len(head) and head[j].startswith("//"):
            j += 1
        assert j >= 2, "header block not found"
        self.old_header_block = head[:j]
        self.old_header_tail = head[j:]
        for ln in header_lines:
            self.new_pairs.append((ln, True))

    # ---- 主装配 ----
    def build(self, sections, annots):
        """sections: va -> lines(SB 插到函数横幅前)；annots: va -> lines(插到关闭横幅后)"""
        starts = self._banner_starts()
        bounds = starts + [len(self.old_lines)]
        # 头块之后的内容
        first_banner = starts[0]
        for ln in self.old_header_tail:
            self.new_pairs.append((ln, False))
        for bi in range(len(starts)):
            s, e = bounds[bi], bounds[bi + 1]
            va = self.va_of_block(s)
            # SECTION 横幅（前补一个空行分隔，避免贴着上一函数）
            if va in sections:
                self.new_pairs.append(("", True))
                for ln in sections[va]:
                    self.new_pairs.append((ln, True))
            for i in range(s, e):
                ln = self.old_lines[i]
                self.new_pairs.append((ln, False))
                # 函数头注释：插在第 3 行横幅（s+2）之后
                if i == s + 2:
                    for ann in annots.get(va, []):
                        self.new_pairs.append((ann, True))
        # 处理 old_lines 中位于最后一个函数块之后的残余行（块切分已覆盖到文件尾，这里只防御）
        return self.new_pairs

    def save(self, out_path):
        text = "\r\n".join(ln for ln, _ in self.new_pairs)
        open(out_path, "w", encoding="utf-8", newline="").write(text)

    # ---- 自验 1：剥离插入行后必须还原旧文件 ----
    def verify_lossless(self):
        starts = self._banner_starts()
        first_banner = starts[0]
        stripped = (
            self.old_header_block
            + [ln for ln, ins in self.new_pairs if not ins]
        )
        expect = self.old_lines
        if stripped != expect:
            # 找出第一处差异便于排错
            for i, (a, b) in enumerate(zip(stripped, expect)):
                if a != b:
                    raise AssertionError(
                        "lossless check failed @%d: %r != %r" % (i, a, b))
            raise AssertionError("lossless check failed: length %d vs %d"
                                 % (len(stripped), len(expect)))
        return True

    # ---- 自验 2：VA 计数 ----
    @staticmethod
    def count_vas(path):
        n = 0
        for ln in open(path, encoding="utf-8"):
            if VA_RE.match(ln.rstrip("\r\n")):
                n += 1
        return n

    # ---- 自验 3：括号配平（剔除 // 行注释后统计；文件内无 /* */ 含括号情形另行核对）----
    @staticmethod
    def braces(path):
        ob = cb = 0
        for ln in open(path, encoding="utf-8"):
            code = re.sub(r"//.*$", "", ln.rstrip("\r\n"))
            ob += code.count("{")
            cb += code.count("}")
        return ob, cb
