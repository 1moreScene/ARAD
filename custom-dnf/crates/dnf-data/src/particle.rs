//! Scripted ActiveStatus PTL emitter subset used by the supplied official assets.
#[derive(Clone, Debug)]
pub struct ParticleScript {
    pub vanish_on_land: bool,
    pub objects: Vec<String>,
    pub pos: [f32; 3],
    pub area: [f32; 3],
    pub axes: [Axis; 3],
    pub frequency: f64,
    pub life_ms: f64,
    pub gravity: f32,
    pub alpha: [f32; 2],
    pub change_ms: f64,
    pub max_time_ms: f64,
    pub max_count: usize,
    pub circle: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Axis {
    pub kind: AxisKind,
    pub first: [f32; 2],
    pub second: [f32; 2],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AxisKind {
    Stop,
    Line,
    Sin,
    Cos,
}
impl Default for Axis {
    fn default() -> Self {
        Self {
            kind: AxisKind::Stop,
            first: [0.0; 2],
            second: [1000.0; 2],
        }
    }
}
pub fn parse(text: &str, directory: &str) -> Result<ParticleScript, String> {
    let mut p = ParticleScript {
        vanish_on_land: false,
        objects: Vec::new(),
        pos: [0.0; 3],
        area: [0.0; 3],
        axes: [Axis::default(); 3],
        frequency: 0.0,
        life_ms: 1000.0,
        gravity: 0.0,
        alpha: [255.0; 2],
        change_ms: 1000.0,
        max_time_ms: 0.0,
        max_count: 0,
        circle: false,
    };
    let mut axis = 0;
    let mut objects = false;
    for raw in text.lines() {
        let line = raw.split("//").next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if objects && line.starts_with('`') {
            if let Some(path) = crate::mob::quoted(line) {
                let mut parts = Vec::new();
                let full = format!("{directory}/{path}");
                for v in full.replace('\\', "/").split('/') {
                    match v {
                        ".." => {
                            parts.pop();
                        }
                        "." | "" => {}
                        _ => parts.push(v.to_string()),
                    }
                }
                p.objects.push(parts.join("/"));
            }
            continue;
        }
        if !line.starts_with('[') {
            continue;
        }
        let end = line.find(']').ok_or("invalid PTL tag")?;
        let tag = &line[1..end];
        let rest = line[end + 1..].trim();
        let values: Vec<f32> = rest
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        let scalar = values.first().copied().unwrap_or(0.0);
        let pair = [scalar, values.get(1).copied().unwrap_or(scalar)];
        match tag {
            "object" => objects = true,
            "/object" => objects = false,
            "x axis" => axis = 0,
            "y axis" => axis = 1,
            "z axis" => axis = 2,
            "move type" => {
                p.axes[axis].kind = match rest {
                    "`[straight line]`" => AxisKind::Line,
                    "`[sin]`" => AxisKind::Sin,
                    "`[cos]`" => AxisKind::Cos,
                    _ => AxisKind::Stop,
                }
            }
            "move variable 1" => p.axes[axis].first = pair,
            "move variable 2" => p.axes[axis].second = pair,
            "creation pos" | "create area size" => {
                let dst = if tag == "creation pos" {
                    &mut p.pos
                } else {
                    &mut p.area
                };
                for (d, v) in dst.iter_mut().zip(values) {
                    *d = v;
                }
            }
            "creation frequency" => p.frequency = scalar as f64,
            "life time" => p.life_ms = scalar as f64,
            "gravity" => p.gravity = scalar,
            "landing type" => p.vanish_on_land = rest == "`[vanish]`",
            "transparent value" => p.alpha = pair,
            "changing term" => p.change_ms = scalar as f64,
            "maximum create time" => p.max_time_ms = scalar as f64,
            "maximum create number" => p.max_count = scalar.max(0.0) as usize,
            "rotation angle range type" => p.circle = rest == "`[circle]`",
            _ => {}
        }
    }
    if p.objects.is_empty()
        || p.life_ms <= 0.0
        || p.frequency < -1.0
        || p.frequency == -1.0 && p.max_count == 0
    {
        return Err("invalid ActiveStatus PTL".into());
    }
    Ok(p)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn official_status_particles_keep_motion_and_instant_burst_count() {
        let Some(dir) = crate::locate_client_dir() else {
            return;
        };
        let pvf = crate::pvf::PvfReader::open(&dir.join("Script.pvf")).unwrap();
        let root = "common/activestatus/particle";
        let text = pvf
            .read_text(&format!("{root}/poisonparticle.ptl"))
            .unwrap();
        let p = parse(&text, root).unwrap();
        assert_eq!(p.frequency, 8.0);
        assert_eq!(p.life_ms, 1600.0);
        assert_eq!(p.axes[0].kind, AxisKind::Sin);
        assert_eq!(
            p.objects[0],
            "common/activestatus/Animation/PoisonParticle1.ani"
        );
        let text = pvf
            .read_text(&format!("{root}/stonecursedestroy.ptl"))
            .unwrap();
        let p = parse(&text, root).unwrap();
        assert_eq!(p.frequency, -1.0);
        assert_eq!(p.max_count, 10);
        assert_eq!(p.objects.len(), 4);
    }
}
