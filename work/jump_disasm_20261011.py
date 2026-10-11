"""Read-only focused PE32 disassembly for jump evidence."""
import struct
import sys
from pathlib import Path
import capstone

exe = Path(__file__).resolve().parents[1] / '40jp-client' / '40jp单机版' / 'ARAD.exe'
data = exe.read_bytes()
pe = struct.unpack_from('<I', data, 0x3C)[0]
nsects = struct.unpack_from('<H', data, pe + 6)[0]
optsz = struct.unpack_from('<H', data, pe + 20)[0]
base = struct.unpack_from('<I', data, pe + 24 + 28)[0]
sects = []
for i in range(nsects):
    s = pe + 24 + optsz + i * 40
    size, rva, raw_size, off = struct.unpack_from('<IIII', data, s + 8)
    sects.append((rva, max(size, raw_size), off))
def read(va, size):
    rva = va - base
    for start, length, off in sects:
        if start <= rva < start + length:
            return data[off + rva - start:off + rva - start + size]
    raise ValueError(hex(va))
md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
start, end = (int(v, 16) for v in sys.argv[1:3])
for ins in md.disasm(read(start, end - start), start):
    print(f'{ins.address:08X} {ins.mnemonic:8} {ins.op_str}')
