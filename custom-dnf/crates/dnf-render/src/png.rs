//! 最小 PNG 写出（RGBA8，非压缩 deflate 存储块）。无第三方依赖。

use crate::framebuffer::Framebuffer;
use std::io::{self, Write};

fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for chunk in data.chunks(5552) {
        for &byte in chunk {
            a += byte as u32;
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    (b << 16) | a
}

/// 把 zlib 流以「存储块」方式写出（有效但未压缩）。
fn zlib_store(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len() + raw.len() / 65535 * 5 + 16);
    out.extend_from_slice(&[0x78, 0x01]); // CMF/FLG：deflate, 32K window, 最快
    let mut pos = 0usize;
    if raw.is_empty() {
        out.extend_from_slice(&[0x01, 0x00, 0x00, 0xFF, 0xFF]);
    }
    while pos < raw.len() {
        let n = (raw.len() - pos).min(65535);
        let last = pos + n == raw.len();
        out.push(if last { 0x01 } else { 0x00 });
        out.extend_from_slice(&(n as u16).to_le_bytes());
        out.extend_from_slice(&(!(n as u16)).to_le_bytes());
        out.extend_from_slice(&raw[pos..pos + n]);
        pos += n;
    }
    out.extend_from_slice(&adler32(raw).to_be_bytes());
    out
}

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, t) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *t = c;
    }
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c = table[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

fn chunk(kind: &[u8; 4], payload: &[u8], out: &mut Vec<u8>) {
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    let mut body = Vec::with_capacity(4 + payload.len());
    body.extend_from_slice(kind);
    body.extend_from_slice(payload);
    out.extend_from_slice(&body);
    out.extend_from_slice(&crc32(&body).to_be_bytes());
}

/// 编码为 PNG 字节流。
pub fn encode(fb: &Framebuffer) -> Vec<u8> {
    let mut raw = Vec::with_capacity(fb.height * (fb.width * 4 + 1));
    for y in 0..fb.height {
        raw.push(0); // filter: None
        let start = y * fb.width * 4;
        raw.extend_from_slice(&fb.data[start..start + fb.width * 4]);
    }

    let mut out = Vec::with_capacity(raw.len() + 1024);
    out.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(fb.width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(fb.height as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8bit, RGBA, no filter/interlace
    chunk(b"IHDR", &ihdr, &mut out);
    chunk(b"IDAT", &zlib_store(&raw), &mut out);
    chunk(b"IEND", &[], &mut out);
    out
}

/// 写 PNG 文件。
pub fn write_file(path: &std::path::Path, fb: &Framebuffer) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let bytes = encode(fb);
    let mut f = std::fs::File::create(path)?;
    f.write_all(&bytes)
}
