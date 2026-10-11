//! Cached official sounds, bounded polyphonic mixer and asynchronous WinMM output.
use dnf_data::audio::{decode_wave, AudioCatalog, Pcm, PreparedAudio, INITIAL_TAGS};
use std::collections::HashMap;
use std::sync::Arc;

pub const RATE: u32 = 44100;
const MAX_VOICES: usize = 32;
const LOOKAHEAD: usize = RATE as usize / 200; // 5ms.
const CEILING: f32 = 0.95;

/// Linked stereo peak limiter. Delay lets us attenuate before a peak arrives;
/// slow gain recovery avoids sample-by-sample hard clipping and channel shifts.
struct Limiter {
    delay: Vec<[f32; 2]>,
    peaks: std::collections::VecDeque<(u64, f32)>,
    cursor: usize,
    sequence: u64,
    gain: f32,
    release: f32,
}
impl Limiter {
    fn new() -> Self {
        Self {
            delay: vec![[0.0; 2]; LOOKAHEAD],
            peaks: std::collections::VecDeque::with_capacity(LOOKAHEAD + 1),
            cursor: 0,
            sequence: 0,
            gain: 1.0,
            release: (-1.0 / (RATE as f32 * 0.08)).exp(),
        }
    }
    fn process(&mut self, input: [f32; 2]) -> [f32; 2] {
        let peak = input[0].abs().max(input[1].abs());
        while self.peaks.back().is_some_and(|&(_, v)| v <= peak) {
            self.peaks.pop_back();
        }
        self.peaks.push_back((self.sequence, peak));
        let earliest = self.sequence.saturating_sub(LOOKAHEAD as u64);
        while self.peaks.front().is_some_and(|&(n, _)| n < earliest) {
            self.peaks.pop_front();
        }
        let maximum = self.peaks.front().map_or(0.0, |&(_, p)| p);
        let target = if maximum > CEILING {
            CEILING / maximum
        } else {
            1.0
        };
        self.gain = target.min(1.0 - (1.0 - self.gain) * self.release);
        let delayed = self.delay[self.cursor];
        self.delay[self.cursor] = input;
        self.cursor = (self.cursor + 1) % LOOKAHEAD;
        self.sequence += 1;
        [delayed[0] * self.gain, delayed[1] * self.gain]
    }
}

pub struct Bank {
    catalog: AudioCatalog,
    samples: HashMap<String, Arc<Pcm>>,
    random: u64,
}
impl Bank {
    pub fn load(audio: &PreparedAudio, seed: u64) -> Result<Self, String> {
        let mut samples = HashMap::new();
        for tag in INITIAL_TAGS {
            let files = audio.catalog.files_for(tag);
            if files.is_empty() {
                return Err(format!("missing audio tag {tag}"));
            }
            for file in files {
                if let std::collections::hash_map::Entry::Vacant(entry) = samples.entry(file) {
                    let file = entry.key();
                    let bytes = audio.packs.read(file).map_err(|e| format!("{file}: {e}"))?;
                    let pcm = decode_wave(&bytes).map_err(|e| format!("{file}: {e}"))?;
                    entry.insert(Arc::new(pcm));
                }
            }
        }
        Ok(Self {
            catalog: audio.catalog.clone(),
            samples,
            random: seed.max(1),
        })
    }
    pub fn len(&self) -> usize {
        self.samples.len()
    }
    fn select(&mut self, tag: &str) -> Option<(Arc<Pcm>, f32, i32)> {
        let state = &mut self.random;
        let effect = self.catalog.resolve(tag, &mut |total| {
            // Rejection sampling avoids modulo bias for xml integer weights.
            let threshold = total.wrapping_neg() % total;
            loop {
                *state ^= *state >> 12;
                *state ^= *state << 25;
                *state ^= *state >> 27;
                let v = state.wrapping_mul(2685821657736338717);
                if v >= threshold {
                    break v % total;
                }
            }
        })?;
        // sub_6611E0: percent*100; sub_65D990: linear (10000+adjust)/10000.
        let adjust = effect.volume_adjust.unwrap_or(0.0);
        let gain = if adjust.is_finite() {
            (1.0 + adjust / 100.0).clamp(0.0, 1.0)
        } else {
            1.0
        };
        Some((
            self.samples.get(&effect.file)?.clone(),
            gain,
            effect.priority.unwrap_or(0),
        ))
    }
    pub fn play(&mut self, tag: &str, mixer: &mut Mixer) -> bool {
        if let Some((pcm, gain, priority)) = self.select(tag) {
            mixer.play(pcm, gain, priority)
        } else {
            false
        }
    }
    pub fn set_loop(&mut self, tag: Option<&'static str>, mixer: &mut Mixer) {
        if mixer.loop_tag == tag {
            return;
        }
        let sound = tag.and_then(|tag| self.loop_sound(tag));
        mixer.set_loop(tag, sound);
    }
    fn loop_sound(&mut self, tag: &str) -> Option<(Arc<Pcm>, f32, i32, f32)> {
        let (pcm, gain, priority) = self.select(tag)?;
        let delay = self.catalog.effects.get(tag).map_or(0.0, |e| e.loop_delay);
        Some((pcm, gain, priority, delay))
    }
}

struct Voice {
    pcm: Arc<Pcm>,
    position: f64,
    gain: f32,
    priority: i32,
    loop_cycle: Option<f64>,
}
pub struct Mixer {
    voices: Vec<Voice>,
    volume: f32,
    limiter: Limiter,
    loop_tag: Option<&'static str>,
    pub peak_voices: usize,
}
impl Mixer {
    pub fn new(volume: f32) -> Self {
        Self {
            voices: Vec::new(),
            volume: volume.clamp(0.0, 1.0),
            limiter: Limiter::new(),
            loop_tag: None,
            peak_voices: 0,
        }
    }
    fn play(&mut self, pcm: Arc<Pcm>, gain: f32, priority: i32) -> bool {
        if self.voices.len() == MAX_VOICES {
            let (index, lowest) = self
                .voices
                .iter()
                .enumerate()
                .min_by_key(|(_, v)| v.priority)
                .unwrap();
            if priority < lowest.priority {
                return false;
            }
            self.voices.remove(index);
        }
        self.voices.push(Voice {
            pcm,
            position: 0.0,
            gain,
            priority,
            loop_cycle: None,
        });
        self.peak_voices = self.peak_voices.max(self.voices.len());
        true
    }
    pub fn stop(&mut self) {
        self.voices.clear();
        self.limiter = Limiter::new();
        self.loop_tag = None;
    }
    fn set_loop(&mut self, tag: Option<&'static str>, sound: Option<(Arc<Pcm>, f32, i32, f32)>) {
        self.voices.retain(|v| v.loop_cycle.is_none());
        self.loop_tag = tag;
        if let Some((pcm, gain, priority, delay)) = sound {
            let cycle = pcm.samples.len() as f64 / pcm.channels as f64
                + delay as f64 * pcm.sample_rate as f64;
            if self.play(pcm, gain, priority) {
                self.voices.last_mut().unwrap().loop_cycle = Some(cycle);
            }
        }
    }
    pub fn render(&mut self, output: &mut [i16]) {
        for stereo in output.chunks_exact_mut(2) {
            let mut sum = [0.0f32; 2];
            for v in &mut self.voices {
                let channels = v.pcm.channels as usize;
                let frames = v.pcm.samples.len() / channels;
                if let Some(cycle) = v.loop_cycle {
                    if v.position >= cycle {
                        v.position %= cycle;
                    }
                }
                let i = v.position as usize;
                if i >= frames {
                    if v.loop_cycle.is_some() {
                        v.position += v.pcm.sample_rate as f64 / RATE as f64;
                    }
                    continue;
                }
                let next = (i + 1).min(frames - 1);
                let t = (v.position - i as f64) as f32;
                for (channel, value) in sum.iter_mut().enumerate() {
                    let c = channel.min(channels - 1);
                    let a = v.pcm.samples[i * channels + c];
                    let b = v.pcm.samples[next * channels + c];
                    *value += (a + (b - a) * t) * v.gain;
                }
                v.position += v.pcm.sample_rate as f64 / RATE as f64;
            }
            // 6dB mix headroom, then a 5ms lookahead limiter at -0.45dBFS.
            let limited = self.limiter.process(sum.map(|s| s * self.volume * 0.5));
            for (value, sample) in stereo.iter_mut().zip(limited) {
                *value = (sample * 32768.0).round() as i16;
            }
        }
        self.voices.retain(|v| {
            v.loop_cycle.is_some()
                || v.position < (v.pcm.samples.len() / v.pcm.channels as usize) as f64
        });
    }
}

#[cfg(windows)]
mod output {
    use super::*;
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, SyncSender, TryRecvError},
    };
    use std::thread::{self, JoinHandle};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[repr(C, packed(2))]
    struct WaveFormat {
        tag: u16,
        channels: u16,
        rate: u32,
        bytes_per_sec: u32,
        align: u16,
        bits: u16,
        extra: u16,
    }
    #[repr(C)]
    struct WaveHdr {
        data: *mut i8,
        length: u32,
        recorded: u32,
        user: usize,
        flags: u32,
        loops: u32,
        next: *mut WaveHdr,
        reserved: usize,
    }
    type Handle = *mut std::ffi::c_void;
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateEventW(
            attributes: *const std::ffi::c_void,
            manual: i32,
            initial: i32,
            name: *const u16,
        ) -> Handle;
        fn SetEvent(event: Handle) -> i32;
        fn WaitForSingleObject(event: Handle, timeout: u32) -> u32;
        fn CloseHandle(handle: Handle) -> i32;
    }
    // Windows event handles can be signaled and waited on across threads.
    // Arc keeps it alive until both the engine and the device have finished.
    struct Event {
        handle: usize,
    }
    impl Event {
        fn new() -> Result<Self, String> {
            let handle = unsafe { CreateEventW(std::ptr::null(), 0, 0, std::ptr::null()) };
            if handle.is_null() {
                Err(std::io::Error::last_os_error().to_string())
            } else {
                Ok(Self {
                    handle: handle as usize,
                })
            }
        }
        fn signal(&self) {
            unsafe {
                SetEvent(self.handle as Handle);
            }
        }
        fn wait(&self) {
            unsafe {
                WaitForSingleObject(self.handle as Handle, 5);
            }
        }
    }
    impl Drop for Event {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.handle as Handle);
            }
        }
    }
    #[link(name = "avrt")]
    extern "system" {
        fn AvSetMmThreadCharacteristicsW(task: *const u16, index: *mut u32) -> Handle;
        fn AvRevertMmThreadCharacteristics(handle: Handle) -> i32;
    }
    struct AudioScheduling(Handle);
    impl AudioScheduling {
        fn enter() -> Self {
            let task: Vec<u16> = "Audio".encode_utf16().chain(Some(0)).collect();
            let mut index = 0;
            Self(unsafe { AvSetMmThreadCharacteristicsW(task.as_ptr(), &mut index) })
        }
    }
    impl Drop for AudioScheduling {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    AvRevertMmThreadCharacteristics(self.0);
                }
            }
        }
    }
    #[link(name = "winmm")]
    extern "system" {
        fn waveOutOpen(
            handle: *mut Handle,
            id: u32,
            format: *const WaveFormat,
            callback: usize,
            instance: usize,
            flags: u32,
        ) -> u32;
        fn waveOutPrepareHeader(handle: Handle, header: *mut WaveHdr, size: u32) -> u32;
        fn waveOutWrite(handle: Handle, header: *mut WaveHdr, size: u32) -> u32;
        fn waveOutUnprepareHeader(handle: Handle, header: *mut WaveHdr, size: u32) -> u32;
        fn waveOutReset(handle: Handle) -> u32;
        fn waveOutClose(handle: Handle) -> u32;
        fn waveOutGetPosition(handle: Handle, time: *mut WaveTime, size: u32) -> u32;
    }
    #[repr(C)]
    struct WaveTime {
        kind: u32,
        value: u32,
        padding: u32,
    }
    fn check(code: u32, operation: &str) -> Result<(), String> {
        if code == 0 {
            Ok(())
        } else {
            Err(format!("{operation}: WinMM {code}"))
        }
    }
    const BLOCK_FRAMES: usize = 441; // 10ms.
                                     // Native uncapped-startup checks observed mapper completions delayed to 121..133ms.
                                     // Keep one extra 40ms margin without changing 10ms blocks or mixer sample timing.
    const BLOCK_COUNT: usize = 16;
    struct Block {
        header: *mut WaveHdr,
        samples: Box<[i16]>,
        queued: bool,
        audible: bool,
    }
    struct Device {
        handle: Handle,
        blocks: Vec<Block>,
        _event: Arc<Event>,
        start: std::time::Instant,
    }
    impl Device {
        fn open(event: Arc<Event>) -> Result<Self, String> {
            let format = WaveFormat {
                tag: 1,
                channels: 2,
                rate: RATE,
                bytes_per_sec: RATE * 4,
                align: 4,
                bits: 16,
                extra: 0,
            };
            let mut handle = std::ptr::null_mut();
            // All WinMM handles, headers and PCM buffers belong to this worker.
            unsafe {
                check(
                    waveOutOpen(&mut handle, u32::MAX, &format, event.handle, 0, 0x50000),
                    "waveOutOpen",
                )?;
            }
            let mut device = Self {
                handle,
                blocks: Vec::new(),
                _event: event,
                start: std::time::Instant::now(),
            };
            for _ in 0..BLOCK_COUNT {
                let mut samples = vec![0i16; BLOCK_FRAMES * 2].into_boxed_slice();
                let header = Box::into_raw(Box::new(WaveHdr {
                    data: samples.as_mut_ptr().cast(),
                    length: (samples.len() * 2) as u32,
                    recorded: 0,
                    user: 0,
                    flags: 0,
                    loops: 0,
                    next: std::ptr::null_mut(),
                    reserved: 0,
                }));
                let result = unsafe {
                    waveOutPrepareHeader(handle, header, std::mem::size_of::<WaveHdr>() as u32)
                };
                if result != 0 {
                    unsafe {
                        drop(Box::from_raw(header));
                    }
                    return Err(format!("waveOutPrepareHeader: WinMM {result}"));
                }
                device.blocks.push(Block {
                    header,
                    samples,
                    queued: false,
                    audible: false,
                });
            }
            Ok(device)
        }
        fn stop(&mut self) -> Result<(), String> {
            unsafe {
                check(waveOutReset(self.handle), "waveOutReset")?;
            }
            for block in &mut self.blocks {
                block.queued = false;
                block.audible = false;
            }
            Ok(())
        }
        fn service(&mut self, mixer: &mut Mixer, stats: &Stats) -> Result<(), String> {
            let all_returned = self.blocks.iter().all(|block| {
                block.queued
                    && unsafe { std::ptr::read_volatile(std::ptr::addr_of!((*block.header).flags)) }
                        & 1
                        != 0
            });
            if all_returned {
                stats.underruns.fetch_add(1, Ordering::Relaxed);
                let mut time = WaveTime {
                    kind: 2,
                    value: 0,
                    padding: 0,
                };
                let result = unsafe {
                    waveOutGetPosition(
                        self.handle,
                        &mut time,
                        std::mem::size_of::<WaveTime>() as u32,
                    )
                };
                eprintln!(
                    "音频队列耗尽: elapsed={:?} position={} kind={} result={} audible={} voices={}",
                    self.start.elapsed(),
                    time.value,
                    time.kind,
                    result,
                    self.blocks.iter().filter(|b| b.audible).count(),
                    mixer.voices.len()
                );
            }
            for block in &mut self.blocks {
                if block.queued {
                    // WinMM documents polling WHDR_DONE; never borrow a queued header/buffer.
                    let flags = unsafe {
                        std::ptr::read_volatile(std::ptr::addr_of!((*block.header).flags))
                    };
                    if flags & 1 == 0 {
                        continue;
                    }
                    stats.completed.fetch_add(1, Ordering::Relaxed);
                    if block.audible {
                        stats.audible_completed.fetch_add(1, Ordering::Relaxed);
                    }
                    block.queued = false;
                }
                mixer.render(&mut block.samples);
                block.audible = block.samples.iter().any(|&s| s != 0);
                unsafe {
                    check(
                        waveOutWrite(
                            self.handle,
                            block.header,
                            std::mem::size_of::<WaveHdr>() as u32,
                        ),
                        "waveOutWrite",
                    )?;
                }
                block.queued = true;
                stats.submitted.fetch_add(1, Ordering::Relaxed);
            }
            stats
                .peak_voices
                .fetch_max(mixer.peak_voices as u64, Ordering::Relaxed);
            Ok(())
        }
    }
    impl Drop for Device {
        fn drop(&mut self) {
            unsafe {
                waveOutReset(self.handle);
            }
            for block in self.blocks.drain(..) {
                let code = unsafe {
                    waveOutUnprepareHeader(
                        self.handle,
                        block.header,
                        std::mem::size_of::<WaveHdr>() as u32,
                    )
                };
                if code == 0 {
                    unsafe {
                        drop(Box::from_raw(block.header));
                    }
                } else {
                    // A broken device must never keep a pointer to freed audio memory.
                    eprintln!("音频设备释放失败 {code}；保留该缓冲直到进程退出");
                    std::mem::forget(block.samples);
                }
            }
            unsafe {
                waveOutClose(self.handle);
            }
        }
    }
    #[derive(Default)]
    pub struct Stats {
        pub requests: AtomicU64,
        pub submitted: AtomicU64,
        pub completed: AtomicU64,
        pub audible_completed: AtomicU64,
        pub peak_voices: AtomicU64,
        pub dropped: AtomicU64,
        pub underruns: AtomicU64,
        pub loops_started: AtomicU64,
    }
    impl Stats {
        pub fn report(&self) -> String {
            format!("audio requests={} submitted={} completed={} audible_completed={} peak_voices={} dropped={} underruns={} loops_started={}",
                self.requests.load(Ordering::Relaxed),self.submitted.load(Ordering::Relaxed),
                self.completed.load(Ordering::Relaxed),self.audible_completed.load(Ordering::Relaxed),
                self.peak_voices.load(Ordering::Relaxed),self.dropped.load(Ordering::Relaxed),self.underruns.load(Ordering::Relaxed),self.loops_started.load(Ordering::Relaxed))
        }
    }
    enum Command {
        Play(Arc<Pcm>, f32, i32, u64),
        Shutdown,
        Loop(Option<&'static str>, Option<(Arc<Pcm>, f32, i32, f32)>, u64),
    }
    pub struct Engine {
        bank: Bank,
        sender: SyncSender<Command>,
        worker: Option<JoinHandle<()>>,
        generation: Arc<AtomicU64>,
        event: Arc<Event>,
        active: bool,
        loop_tag: Option<&'static str>,
        pub stats: Arc<Stats>,
    }
    impl Engine {
        pub fn start(audio: &PreparedAudio, volume: f32) -> Result<Self, String> {
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            let bank = Bank::load(audio, seed)?;
            let (sender, receiver) = mpsc::sync_channel(256);
            let (ready_send, ready_recv) = mpsc::sync_channel(1);
            let stats = Arc::new(Stats::default());
            let generation = Arc::new(AtomicU64::new(0));
            let event = Arc::new(Event::new()?);
            let s = stats.clone();
            let g = generation.clone();
            let wake = event.clone();
            let worker = thread::Builder::new()
                .name("dnf-audio".into())
                .spawn(move || {
                    let _scheduling = AudioScheduling::enter();
                    let mut device = match Device::open(wake.clone()) {
                        Ok(device) => {
                            let _ = ready_send.send(Ok(()));
                            device
                        }
                        Err(e) => {
                            let _ = ready_send.send(Err(e));
                            return;
                        }
                    };
                    let mut mixer = Mixer::new(volume);
                    let mut epoch = 0;
                    loop {
                        let now = g.load(Ordering::Acquire);
                        if now != epoch {
                            mixer.stop();
                            if let Err(e) = device.stop() {
                                eprintln!("音频停止失败: {e}");
                                break;
                            }
                            epoch = now;
                        }
                        let mut shutdown = false;
                        // Bounded work prevents a flood of commands starving the device.
                        for _ in 0..256 {
                            match receiver.try_recv() {
                                Ok(Command::Play(pcm, gain, priority, issued)) => {
                                    if issued == epoch {
                                        if mixer.play(pcm, gain, priority) {
                                            s.requests.fetch_add(1, Ordering::Relaxed);
                                        } else {
                                            s.dropped.fetch_add(1, Ordering::Relaxed);
                                        }
                                    }
                                }
                                Ok(Command::Shutdown) | Err(TryRecvError::Disconnected) => {
                                    shutdown = true;
                                    break;
                                }
                                Ok(Command::Loop(tag, sound, issued)) => {
                                    if issued == epoch {
                                        if sound.is_some() {
                                            s.loops_started.fetch_add(1, Ordering::Relaxed);
                                        }
                                        mixer.set_loop(tag, sound);
                                    }
                                }
                                Err(TryRecvError::Empty) => break,
                            }
                        }
                        if shutdown {
                            break;
                        }
                        if let Err(e) = device.service(&mut mixer, &s) {
                            eprintln!("音频播放失败: {e}");
                            break;
                        }
                        wake.wait();
                    }
                })
                .map_err(|e| e.to_string())?;
            match ready_recv.recv().map_err(|e| e.to_string())? {
                Ok(()) => Ok(Self {
                    bank,
                    sender,
                    worker: Some(worker),
                    generation,
                    event,
                    active: true,
                    loop_tag: None,
                    stats,
                }),
                Err(error) => {
                    let _ = worker.join();
                    Err(error)
                }
            }
        }
        pub fn cached_sounds(&self) -> usize {
            self.bank.len()
        }
        pub fn set_active(&mut self, active: bool) {
            if self.active && !active {
                self.generation.fetch_add(1, Ordering::AcqRel);
                self.loop_tag = None;
                self.event.signal();
            }
            self.active = active;
        }
        pub fn play(&mut self, tags: &[&str]) {
            if !self.active {
                return;
            }
            for tag in tags {
                if let Some((pcm, gain, priority)) = self.bank.select(tag) {
                    let generation = self.generation.load(Ordering::Acquire);
                    if self
                        .sender
                        .try_send(Command::Play(pcm, gain, priority, generation))
                        .is_err()
                    {
                        self.stats.dropped.fetch_add(1, Ordering::Relaxed);
                    }
                } else {
                    eprintln!("缺少可播放音效: {tag}");
                }
            }
            if !tags.is_empty() {
                self.event.signal();
            }
        }
    }
    impl Engine {
        pub fn set_loop(&mut self, tag: Option<&'static str>) {
            if !self.active || self.loop_tag == tag {
                return;
            }
            let sound = tag.and_then(|tag| self.bank.loop_sound(tag));
            let epoch = self.generation.load(Ordering::Acquire);
            if self
                .sender
                .try_send(Command::Loop(tag, sound, epoch))
                .is_ok()
            {
                self.loop_tag = tag;
                self.event.signal();
            } else {
                self.stats.dropped.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    impl Drop for Engine {
        fn drop(&mut self) {
            let _ = self.sender.send(Command::Shutdown);
            self.event.signal();
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
            println!("{}", self.stats.report());
        }
    }
}
#[cfg(windows)]
pub use output::Engine;

/// Export a deterministic sound track for verifying game timing without an output device.
pub fn write_wave(path: &std::path::Path, samples: &[i16]) -> Result<(), String> {
    use std::io::Write;
    let size = u32::try_from(samples.len() * 2).map_err(|_| "audio export too large")?;
    let riff = size.checked_add(36).ok_or("audio export too large")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut f = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut data = Vec::with_capacity(44 + size as usize);
    data.extend_from_slice(b"RIFF");
    data.extend_from_slice(&riff.to_le_bytes());
    data.extend_from_slice(b"WAVEfmt ");
    data.extend_from_slice(&16u32.to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&2u16.to_le_bytes());
    data.extend_from_slice(&RATE.to_le_bytes());
    data.extend_from_slice(&(RATE * 4).to_le_bytes());
    data.extend_from_slice(&4u16.to_le_bytes());
    data.extend_from_slice(&16u16.to_le_bytes());
    data.extend_from_slice(b"data");
    data.extend_from_slice(&size.to_le_bytes());
    for sample in samples {
        data.extend_from_slice(&sample.to_le_bytes());
    }
    f.write_all(&data).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pcm(rate: u32, channels: u16, samples: Vec<f32>) -> Arc<Pcm> {
        Arc::new(Pcm {
            sample_rate: rate,
            channels,
            samples,
        })
    }
    #[test]
    fn movement_loop_keeps_one_voice_repeats_delay_and_stops() {
        let mut m = Mixer::new(1.0);
        let p = pcm(RATE, 1, vec![0.5; 4]);
        m.set_loop(Some("walk"), Some((p.clone(), 1.0, 0, 4.0 / RATE as f32)));
        let mut out = vec![0; (LOOKAHEAD + 20) * 2];
        m.render(&mut out);
        assert_eq!(m.voices.len(), 1);
        let signal = &out[LOOKAHEAD * 2..];
        assert_eq!(&signal[..8], &[8192; 8]);
        assert_eq!(&signal[8..16], &[0; 8]);
        assert_eq!(&signal[16..24], &[8192; 8]);
        m.set_loop(Some("run"), Some((p, 1.0, 0, 0.0)));
        assert_eq!(m.voices.len(), 1);
        m.set_loop(None, None);
        assert!(m.voices.is_empty());
        m.render(&mut out);
        m.render(&mut out);
        assert!(out.iter().all(|&s| s == 0));
    }
    #[test]
    fn overlapping_voices_sum_with_headroom_and_stop() {
        let mut m = Mixer::new(1.0);
        let p = pcm(RATE, 1, vec![0.75; 4]);
        m.play(p.clone(), 1.0, 0);
        m.play(p, 1.0, 0);
        let mut out = vec![0; (LOOKAHEAD + 4) * 2];
        m.render(&mut out);
        assert_eq!(&out[LOOKAHEAD * 2..], &[24576; 8]);
        assert_eq!(m.peak_voices, 2);
        m.stop();
        m.render(&mut out);
        assert!(out.iter().all(|&s| s == 0));
    }
    #[test]
    fn stereo_resampling_volume_and_natural_end() {
        let mut m = Mixer::new(0.5);
        m.play(pcm(RATE / 2, 2, vec![0.0, 0.0, 1.0, -1.0]), 1.0, 0);
        let mut out = vec![0; (LOOKAHEAD + 5) * 2];
        m.render(&mut out);
        assert_eq!(
            &out[LOOKAHEAD * 2..],
            &[0, 0, 4096, -4096, 8192, -8192, 8192, -8192, 0, 0]
        );
        assert!(m.voices.is_empty());
    }
    #[test]
    fn priority_and_voice_budget_are_bounded() {
        let mut m = Mixer::new(1.0);
        let p = pcm(RATE, 1, vec![0.1; 4]);
        for _ in 0..MAX_VOICES {
            assert!(m.play(p.clone(), 1.0, 5));
        }
        assert!(!m.play(p.clone(), 1.0, 4));
        assert!(m.play(p, 1.0, 6));
        assert_eq!(m.voices.len(), MAX_VOICES);
        assert_eq!(m.peak_voices, MAX_VOICES);
    }
    #[test]
    fn limiter_prevents_overload_without_flattening_the_waveform() {
        let mut m = Mixer::new(1.0);
        let samples = (0..2000)
            .map(|i| (i as f32 * 0.07).sin())
            .collect::<Vec<_>>();
        let p = pcm(RATE, 1, samples);
        for _ in 0..MAX_VOICES {
            m.play(p.clone(), 1.0, 0);
        }
        let mut out = vec![0; (2000 + LOOKAHEAD) * 2];
        m.render(&mut out);
        assert!(out.iter().all(|&s| s.abs() <= 31131));
        assert!(out.iter().any(|&s| s.abs() > 30000));
        assert_eq!(
            out.iter().filter(|&&s| s == 32767 || s == -32768).count(),
            0
        );
        // A clipped sum produces flat runs near the rails. The limited sine varies.
        assert!(
            out[LOOKAHEAD * 2..]
                .chunks_exact(2)
                .map(|f| f[0])
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                > 1000
        );
    }
    #[test]
    fn device_block_boundaries_do_not_reset_or_duplicate_samples() {
        let p = pcm(
            RATE,
            2,
            (0..5000).map(|i| (i as f32 * 0.05).sin() * 0.9).collect(),
        );
        let mut a = Mixer::new(1.0);
        let mut b = Mixer::new(1.0);
        a.play(p.clone(), 1.0, 0);
        b.play(p, 1.0, 0);
        let mut whole = vec![0; 6000];
        let mut blocks = vec![0; 6000];
        a.render(&mut whole);
        for block in blocks.chunks_mut(882) {
            b.render(block);
        }
        assert_eq!(whole, blocks);
    }
}
