use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use uuid::Uuid;

struct CapturedAudio {
  samples: Vec<f32>,
  sample_rate: u32,
  channels: u16,
}

pub struct RecordingSession {
  pub id: Uuid,
  started: std::time::Instant,
  stop_tx: mpsc::Sender<()>,
  worker: Option<JoinHandle<Result<CapturedAudio>>>,
}

impl RecordingSession {
  pub fn begin() -> Result<Self> {
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (init_tx, init_rx) = mpsc::channel::<Result<()>>();

    let worker = std::thread::spawn(move || {
      let host = cpal::default_host();
      let device = host
        .default_input_device()
        .context("no input microphone device found")?;

      let supported = device
        .default_input_config()
        .context("failed to query default input config")?;
      let sample_format = supported.sample_format();
      let config: cpal::StreamConfig = supported.into();
      let sample_rate = config.sample_rate.0;
      let channels = config.channels;

      let samples = Arc::new(Mutex::new(Vec::<f32>::new()));
      let err_fn = |err| eprintln!("audio stream error: {err}");

      let stream = match sample_format {
        cpal::SampleFormat::F32 => {
          let write_buf = Arc::clone(&samples);
          device.build_input_stream(
            &config,
            move |data: &[f32], _| {
              write_buf.lock().extend_from_slice(data);
            },
            err_fn,
            None,
          )?
        }
        cpal::SampleFormat::I16 => {
          let write_buf = Arc::clone(&samples);
          device.build_input_stream(
            &config,
            move |data: &[i16], _| {
              let mut guard = write_buf.lock();
              guard.extend(data.iter().map(|s| *s as f32 / i16::MAX as f32));
            },
            err_fn,
            None,
          )?
        }
        cpal::SampleFormat::U16 => {
          let write_buf = Arc::clone(&samples);
          device.build_input_stream(
            &config,
            move |data: &[u16], _| {
              let mut guard = write_buf.lock();
              guard.extend(data.iter().map(|s| (*s as f32 / u16::MAX as f32) * 2.0 - 1.0));
            },
            err_fn,
            None,
          )?
        }
        _ => return Err(anyhow!("unsupported microphone sample format")),
      };

      stream.play().context("failed to start microphone stream")?;
      let _ = init_tx.send(Ok(()));

      let _ = stop_rx.recv();
      drop(stream);
      let captured_samples = samples.lock().clone();

      Ok(CapturedAudio {
        samples: captured_samples,
        sample_rate,
        channels,
      })
    });

    init_rx
      .recv()
      .map_err(|_| anyhow!("failed to initialize recording thread"))??;

    Ok(Self {
      id: Uuid::new_v4(),
      started: std::time::Instant::now(),
      stop_tx,
      worker: Some(worker),
    })
  }

  pub fn elapsed_ms(&self) -> u128 {
    self.started.elapsed().as_millis()
  }
}

fn downmix_and_resample(input: &[f32], channels: u16, in_rate: u32, out_rate: u32) -> Vec<f32> {
  if input.is_empty() {
    return Vec::new();
  }

  let mono = if channels <= 1 {
    input.to_vec()
  } else {
    let mut out = Vec::with_capacity(input.len() / channels as usize);
    for frame in input.chunks_exact(channels as usize) {
      let sum: f32 = frame.iter().copied().sum();
      out.push(sum / channels as f32);
    }
    out
  };

  if in_rate == out_rate {
    return mono;
  }

  let ratio = in_rate as f64 / out_rate as f64;
  let out_len = ((mono.len() as f64) / ratio).max(1.0) as usize;
  let mut out = Vec::with_capacity(out_len);

  for i in 0..out_len {
    let src_pos = i as f64 * ratio;
    let idx = src_pos.floor() as usize;
    let frac = (src_pos - idx as f64) as f32;
    let a = *mono.get(idx).unwrap_or(&0.0);
    let b = *mono.get(idx + 1).unwrap_or(&a);
    out.push(a + (b - a) * frac);
  }

  out
}

pub async fn finalize_capture(mut session: RecordingSession) -> Result<Vec<f32>> {
  let _ = session.stop_tx.send(());
  let worker = session
    .worker
    .take()
    .ok_or_else(|| anyhow!("recording worker handle missing"))?;

  let captured = worker
    .join()
    .map_err(|_| anyhow!("recording worker panicked"))??;

  if captured.samples.is_empty() {
    return Err(anyhow!("no microphone audio captured"));
  }

  Ok(downmix_and_resample(
    &captured.samples,
    captured.channels,
    captured.sample_rate,
    16_000,
  ))
}
