//! Bounded frame telemetry; FPS is presentation throughput, CPU benchmarks are separate.
use std::collections::VecDeque;
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameTime {
    pub frame: f64,
    pub logic: f64,
    pub render: f64,
    pub convert: f64,
    pub scale: f64,
    pub gdi: f64,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Snapshot {
    pub avg: FrameTime,
    pub p95: f64,
    pub p99: f64,
    pub slow_frames: usize,
    pub samples: usize,
}
#[derive(Default)]
pub struct Monitor {
    samples: VecDeque<FrameTime>,
}
impl Monitor {
    pub fn push(&mut self, time: FrameTime) {
        if self.samples.len() == 240 {
            self.samples.pop_front();
        }
        self.samples.push_back(time);
    }
    pub fn snapshot(&self) -> Snapshot {
        if self.samples.is_empty() {
            return Snapshot::default();
        }
        let mut avg = FrameTime::default();
        let n = self.samples.len() as f64;
        for s in &self.samples {
            avg.frame += s.frame / n;
            avg.logic += s.logic / n;
            avg.render += s.render / n;
            avg.convert += s.convert / n;
            avg.scale += s.scale / n;
            avg.gdi += s.gdi / n;
        }
        let mut times: Vec<f64> = self.samples.iter().map(|s| s.frame).collect();
        times.sort_by(f64::total_cmp);
        Snapshot {
            avg,
            p95: times[((n * 0.95).ceil() as usize).saturating_sub(1)],
            p99: times[((n * 0.99).ceil() as usize).saturating_sub(1)],
            slow_frames: times.iter().filter(|&&t| t > 16.667).count(),
            samples: times.len(),
        }
    }
}
pub fn ms(start: std::time::Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_monitor_reports_percentiles_and_separates_stage_costs() {
        let mut m = Monitor::default();
        for i in 1..=250 {
            m.push(FrameTime {
                frame: i as f64,
                render: 1.25,
                scale: 0.75,
                ..Default::default()
            });
        }
        let p = m.snapshot();
        assert_eq!(p.samples, 240);
        assert_eq!(p.p95, 238.0);
        assert_eq!(p.p99, 248.0);
        assert_eq!(p.slow_frames, 234);
        assert!((p.avg.render - 1.25).abs() < 1e-9);
        assert!((p.avg.scale - 0.75).abs() < 1e-9);
    }
}

pub fn benchmark(data: &crate::game::GameData, out: &std::path::Path) -> Result<(), String> {
    use crate::{
        debug_ui::Control,
        script::{self, ScriptKind},
    };
    use dnf_core::{protection::Protection, KeyMap};
    use dnf_render::{framebuffer::Framebuffer, sampling::LinearScaler};
    use std::time::Instant;
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut csv = String::from(
        "case,monsters,armor,mean_render_ms,p95_render_ms,mean_scale_ms,p95_scale_ms,cpu_fps\n",
    );
    for (name, count, armor, panel) in [
        ("normal_1", 1, false, false),
        ("normal_6", 6, false, false),
        ("normal_32", 32, false, false),
        ("armor_32", 32, true, false),
        ("debug_panel", 6, false, true),
        ("grab_hold_6", 6, false, false),
        ("grab_burst_32", 32, false, false),
        ("active_status_1", 1, false, false),
        ("active_status_32", 32, false, false),
        ("active_status_panel", 6, false, true),
    ] {
        let mut g = data.build_game(KeyMap::official_default())?;
        g.actor.teleport(320, 329);
        g.apply_control(Control::ResetMonsters);
        g.set_monster_count(count);
        if armor {
            g.apply_control(Control::MonsterState {
                target: None,
                state: Protection::SuperArmor,
            });
        }
        g.debug_panel.open = panel;
        let grab_case = name.starts_with("grab_");
        if grab_case {
            g.apply_control(Control::MonsterSet {
                target: None,
                field: 1,
                value: 5000,
            });
            g.apply_control(Control::Respawn(None));
            g.begin_input();
            g.set_key(g.keymap.get(dnf_core::keymap::Command::SkillSlot3), true);
            g.step();
            let mut caught = false;
            for _ in 0..100 {
                g.begin_input();
                g.step();
                caught |= g.grab_skill.target().is_some();
                if name == "grab_hold_6" && caught {
                    break;
                }
                if name == "grab_burst_32" && caught && g.grab_skill.target().is_none() {
                    break;
                }
            }
            if !caught {
                return Err(format!("{name}: grab benchmark did not capture"));
            }
        }
        let active_case = name.starts_with("active_status_");
        if active_case {
            g.apply_control(Control::MonsterSet {
                target: None,
                field: 1,
                value: 50000,
            });
            g.apply_control(Control::Respawn(None));
            for kind in dnf_core::active_status::ActiveStatus::ALL
                .into_iter()
                .filter(|k| k.implemented() && *k != dnf_core::active_status::ActiveStatus::Sleep)
            {
                g.apply_monster_status(None, kind, 8);
            }
            g.debug_panel.open = false;
            for _ in 0..42 {
                g.begin_input();
                g.step();
            }
            g.debug_panel.open = panel;
            if panel {
                g.debug_panel.tab = 1;
                g.debug_panel.monster_page = 1;
                g.debug_panel.status_dropdown = true;
            }
        }
        let mut fb = Framebuffer::default();
        let mut scaled = Vec::new();
        let scaler = LinearScaler::new((640, 480), (800, 600));
        let mut render = Monitor::default();
        let mut scale = Monitor::default();
        for i in 0..360 {
            if !grab_case && !active_case && i % 2 == 0 {
                g.begin_input();
                for key in script::sample(ScriptKind::All, (i / 2) as u64, &g.keymap) {
                    g.set_key(key, true);
                }
                g.step();
            }
            let start = Instant::now();
            g.render(data, &mut fb);
            let r = ms(start);
            let start = Instant::now();
            scaler.render(&fb.data, &mut scaled);
            let s = ms(start);
            std::hint::black_box(&scaled);
            if i >= 120 {
                render.push(FrameTime {
                    frame: r,
                    ..Default::default()
                });
                scale.push(FrameTime {
                    frame: s,
                    ..Default::default()
                });
            }
        }
        let r = render.snapshot();
        let s = scale.snapshot();
        let fps = 1000.0 / (r.avg.frame + s.avg.frame).max(0.001);
        csv.push_str(&format!(
            "{name},{count},{armor},{:.4},{:.4},{:.4},{:.4},{fps:.1}\n",
            r.avg.frame, r.p95, s.avg.frame, s.p95
        ));
        println!("CPU BENCH {name}: render {:.3}ms p95 {:.3}ms; scale {:.3}ms p95 {:.3}ms; CPU-only {fps:.1}fps",r.avg.frame,r.p95,s.avg.frame,s.p95);
    }
    std::fs::write(out.join("performance.csv"), csv).map_err(|e| e.to_string())?;
    Ok(())
}
