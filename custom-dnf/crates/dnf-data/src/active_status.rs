//! Official 40JP PE tuning at6CB220/6CB268/6CB2F0. id10/14 are zero/no-op.
#[derive(Clone, Debug)]
pub struct StatusTable {
    pub duration: [i32; 17],
    pub power: [[f32; 2]; 17],
    pub secondary: [[f32; 2]; 17],
}
pub fn pe_bytes(b: &[u8], address: u32, len: usize) -> Result<&[u8], String> {
    let u16_at = |p: usize| {
        b.get(p..p + 2)
            .map(|s| u16::from_le_bytes(s.try_into().unwrap()))
            .ok_or("truncated PE".to_string())
    };
    let u32_at = |p: usize| {
        b.get(p..p + 4)
            .map(|s| u32::from_le_bytes(s.try_into().unwrap()))
            .ok_or("truncated PE".to_string())
    };
    let h = u32_at(60)? as usize;
    let base = u32_at(h + 52)?;
    let sections = u16_at(h + 6)? as usize;
    let start = h + 24 + u16_at(h + 20)? as usize;
    for i in 0..sections {
        let p = start + 40 * i;
        let va = base + u32_at(p + 12)?;
        let size = u32_at(p + 16)?;
        if address >= va && address as u64 + len as u64 <= va as u64 + size as u64 {
            let off = u32_at(p + 20)? as usize + (address - va) as usize;
            return b.get(off..off + len).ok_or("PE section truncated".into());
        }
    }
    Err(format!("PE address unavailable {address:X}"))
}
impl StatusTable {
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let b = std::fs::read(path).map_err(|e| e.to_string())?;
        let mut out = Self {
            duration: [0; 17],
            power: [[0.0; 2]; 17],
            secondary: [[0.0; 2]; 17],
        };
        for i in 0..17 {
            out.duration[i] = i32::from_le_bytes(
                pe_bytes(&b, 0x6CB220 + i as u32 * 4, 4)?
                    .try_into()
                    .unwrap(),
            );
            for j in 0..2 {
                out.power[i][j] = f32::from_le_bytes(
                    pe_bytes(&b, 0x6CB268 + i as u32 * 8 + j as u32 * 4, 4)?
                        .try_into()
                        .unwrap(),
                );
                out.secondary[i][j] = f32::from_le_bytes(
                    pe_bytes(&b, 0x6CB2F0 + i as u32 * 8 + j as u32 * 4, 4)?
                        .try_into()
                        .unwrap(),
                );
            }
            if !(0..=60).contains(&out.duration[i])
                || out.power[i]
                    .iter()
                    .chain(out.secondary[i].iter())
                    .any(|v| !v.is_finite() || *v < 0.0 || *v > 10000.0)
            {
                return Err("unsupported ActiveStatus PE table".into());
            }
        }
        Ok(out)
    }
    pub fn parameters(&self, id: usize, level: u8) -> (f64, f64, f64) {
        let t = (level.clamp(1, 40) - 1) as f64 / 39.0;
        let lerp = |v: [f32; 2]| v[0] as f64 + (v[1] as f64 - v[0] as f64) * t;
        let power = lerp(self.power[id]);
        let duration = if self.duration[id] > 0 {
            self.duration[id] as f64 * 1000.0
        } else {
            (power * 1000.0).trunc()
        };
        (duration, power, lerp(self.secondary[id]))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn official_levels_and_reserved_slots_are_read_from_pe() {
        let Some(dir) = crate::locate_client_dir() else {
            return;
        };
        let table = StatusTable::load(&dir.join("ARAD.exe")).unwrap();
        assert_eq!(table.parameters(2, 1), (7000.0, 6.0, 0.0));
        assert_eq!(table.parameters(6, 40), (15000.0, 600.0, 0.0));
        assert_eq!(table.parameters(7, 1), (4000.0, 4.0, 120.0));
        assert_eq!(table.parameters(10, 40), (0.0, 0.0, 0.0));
        assert_eq!(table.parameters(14, 40), (0.0, 0.0, 0.0));
        assert_eq!(table.parameters(1, 2).0, 2717.0);
    }
}
