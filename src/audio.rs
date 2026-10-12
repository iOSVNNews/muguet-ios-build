// muguet - audio.rs
// Constantin Clerc (c22dev) - 0.1

use crate::RT;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};

static UNDERRUN_FRAMES: AtomicU64 = AtomicU64::new(0);
static CALLBACKS: AtomicU64 = AtomicU64::new(0);

static STARTED: AtomicBool = AtomicBool::new(false);

pub fn start(ring: u64, capacity: u32) {
    if STARTED.swap(true, Ordering::SeqCst) || std::env::var("MUGUET_MUTE").is_ok() {
        return;
    }

    std::thread::Builder::new()
        .name("audio".into())
        .spawn(move || match play(ring, capacity) {
            Ok(stream) => {
                eprintln!("[audio] CoreAudio output started (ring 0x{ring:x}, {capacity} frames)");
                let _keep = stream;
                loop { std::thread::park(); }
            }
            Err(e) => eprintln!("[audio] no output: {e:#}"),
        })
        .expect("spawn audio thread");
}

fn play(ring: u64, capacity: u32) -> anyhow::Result<cpal::Stream> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or_else(|| anyhow::anyhow!("no default output device"))?;
    let config = cpal::StreamConfig { channels: 2, sample_rate: 48000, buffer_size: cpal::BufferSize::Default };
    let (base, _) = crate::mem_base_len(&RT.get().unwrap().mem);
    let base = base as usize;
    let hdr = move |i: usize| -> &'static AtomicI32 { unsafe { &*((base + ring as usize + 4 * i) as *const AtomicI32) } };
    let data = base + ring as usize + 64;
    let mask = capacity.wrapping_sub(1);
    let stream = device.build_output_stream(
        config,
        move |out: &mut [f32], _| {
            let frames = out.len() / 2;
            let read = hdr(1).load(Ordering::SeqCst) as u32;
            let write = hdr(0).load(Ordering::SeqCst) as u32;
            let take = (write.wrapping_sub(read) as usize).min(frames);
            for i in 0..take {
                let at = ((read.wrapping_add(i as u32) & mask) * 2) as usize;
                unsafe {
                    let p = (data as *const f32).add(at);
                    out[2 * i] = *p;
                    out[2 * i + 1] = *p.add(1);
                }
            }
            for s in &mut out[take * 2..] { *s = 0.0; }
            UNDERRUN_FRAMES.fetch_add((frames - take) as u64, Ordering::Relaxed);
            CALLBACKS.fetch_add(1, Ordering::Relaxed);
            hdr(1).store(read.wrapping_add(take as u32) as i32, Ordering::SeqCst);
            hdr(3).fetch_add(1, Ordering::SeqCst);
        },
        |e| eprintln!("[audio] stream error: {e}"),
        None,
    )?;
    stream.play()?;

    std::thread::spawn(move || {
        let mut last_w = hdr(0).load(Ordering::SeqCst) as u32;
        let mut last_u = 0;
        let mut last_c = 0;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));
            let w = hdr(0).load(Ordering::SeqCst) as u32;
            let (u, c) = (UNDERRUN_FRAMES.load(Ordering::Relaxed), CALLBACKS.load(Ordering::Relaxed));
            eprintln!("[audio] mixer produced {:.0} frames/s (needs 48000), underrun {:.0} frames/s, {} callbacks/s, ring fill {}",
                w.wrapping_sub(last_w) as f64 / 5.0, (u - last_u) as f64 / 5.0, (c - last_c) / 5, w.wrapping_sub(hdr(1).load(Ordering::SeqCst) as u32));
            last_w = w; last_u = u; last_c = c;
        }
    });
    Ok(stream)
}
