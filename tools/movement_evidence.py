"""Read-only PE32 evidence for the official movement/input chain (requires capstone).

python tools/movement_evidence.py table 6C490C 31
python tools/movement_evidence.py disasm 437940 437E70
python tools/movement_evidence.py refs 4C0140
"""
import hashlib
import struct
import sys
from pathlib import Path
import capstone

EXE = Path(__file__).resolve().parents[1] / "40jp-client/40jp单机版/ARAD.exe"
blob = EXE.read_bytes()
pe = struct.unpack_from("<I", blob, 0x3C)[0]
base = struct.unpack_from("<I", blob, pe + 52)[0]
sections = []
for i in range(struct.unpack_from("<H", blob, pe + 6)[0]):
    off = pe + 24 + struct.unpack_from("<H", blob, pe + 20)[0] + 40 * i
    size, rva, raw_size, raw = struct.unpack_from("<IIII", blob, off + 8)
    flags = struct.unpack_from("<I", blob, off + 36)[0]
    sections.append((base + rva, size, raw_size, raw, flags))


def read(va, size):
    for start, _, raw_size, off, _ in sections:
        if start <= va and va + size <= start + raw_size:
            return blob[off + va - start:off + va - start + size]
    raise ValueError(f"unmapped file bytes: {va:X}+{size:X}")


md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
md.detail = True
if __name__ == "__main__":
    print(f"ARAD.exe size={len(blob)} sha256={hashlib.sha256(blob).hexdigest()}")
    mode = sys.argv[1]
    va = int(sys.argv[2], 16)
    if mode == "table":
        for i in range(int(sys.argv[3])):
            print(f"+{i*4:03X} {struct.unpack('<I', read(va+i*4,4))[0]:08X}")
    elif mode == "disasm":
        end = int(sys.argv[3], 16)
        for ins in md.disasm(read(va, end - va), va):
            print(f"{ins.address:08X} {ins.bytes.hex():20} {ins.mnemonic:8} {ins.op_str}")
    elif mode == "refs":
        for start, _, size, off, flags in sections:
            if not flags & 0x20000000:
                continue
            # skipdata is needed across padding/data embedded in .text.
            md.skipdata = True
            for ins in md.disasm(blob[off:off+size], start):
                if not ins.id:
                    continue
                if any((op.type == capstone.x86.X86_OP_IMM and op.imm == va)
                       or (op.type == capstone.x86.X86_OP_MEM and op.mem.disp == va)
                       for op in ins.operands):
                    print(f"{ins.address:08X} {ins.mnemonic:8} {ins.op_str}")
    elif mode == "stores":
        # Known x86 MOV r/m32, imm32 with disp32, independent of alignment in .text.
        needle = struct.pack("<I", va)
        for start, _, size, off, flags in sections:
            if not flags & 0x20000000:
                continue
            chunk = blob[off:off+size]
            pos = chunk.find(needle)
            while pos >= 0:
                if pos >= 2 and chunk[pos-2] == 0xC7 and chunk[pos-1] in range(0x80, 0x88):
                    for ins in md.disasm(chunk[pos-2:pos+8], start+pos-2):
                        print(f"{ins.address:08X} {ins.mnemonic:8} {ins.op_str}")
                pos = chunk.find(needle, pos+1)
    else:
        raise ValueError(mode)
