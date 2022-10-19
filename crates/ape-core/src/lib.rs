pub mod dsp;
pub mod engine;
pub mod export;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crate::engine::stream_setup_for_device;
use color_eyre::eyre;
use cpal::traits::StreamTrait;
use engine::host_device_setup;

// Reexports
pub use color_eyre;
pub use cpal;
pub use hound;
pub use tracing;

pub enum AudioOutput {
    Wav(WavOutput),
    Direct(DirectOutput),
}

impl AudioOutput {
    pub fn sample_rate(&self) -> u32 {
        match &self {
            Self::Wav(params) => params.spec.sample_rate,
            Self::Direct(params) => params.config.sample_rate().0,
        }
    }

    pub fn new_direct() -> eyre::Result<Self> {
        let (_host, device, config) = host_device_setup()?;
        Ok(Self::Direct(DirectOutput { device, config }))
    }
}

pub struct WavOutput {
    pub path: PathBuf,
    pub spec: hound::WavSpec,
    pub duration: usize,
}

pub struct DirectOutput {
    pub device: cpal::Device,
    pub config: cpal::SupportedStreamConfig,
}

fn stream_loop(
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
    sample_fn: impl FnMut() -> Vec<f32> + Send + 'static,
) -> eyre::Result<()> {
    let stream = stream_setup_for_device(device, config, sample_fn)?;
    stream.play()?;

    let running = Arc::new(AtomicBool::new(true));
    let handler_running = Arc::clone(&running);
    ctrlc::set_handler(move || {
        handler_running.store(false, Ordering::SeqCst);
    })?;

    println!("Waiting for CTRL+C to quit ...");
    while running.load(Ordering::SeqCst) {}
    println!("Received CTRL+C, terminating.");

    Ok(())
}

fn stream_loop_spinlock(
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
    sample_fn: impl FnMut() -> Vec<f32> + Send + 'static,
    running: Arc<AtomicBool>,
) -> eyre::Result<()> {
    let stream = stream_setup_for_device(device, config, sample_fn)?;
    stream.play()?;

    while running.load(Ordering::Relaxed) {
        thread::yield_now();
    }

    Ok(())
}

pub fn process_stream(
    output: AudioOutput,
    sample_fn: impl FnMut() -> Vec<f32> + Send + 'static,
) -> eyre::Result<()> {
    match output {
        AudioOutput::Wav(params) => {
            export::export_to_wav(&params.path, params.spec, params.duration, sample_fn)
        }
        AudioOutput::Direct(params) => stream_loop(params.device, params.config, sample_fn),
    }
}

pub fn start_stream_thread(
    output: AudioOutput,
    sample_fn: impl FnMut() -> Vec<f32> + Send + 'static,
    running: Arc<AtomicBool>,
) -> eyre::Result<JoinHandle<()>> {
    Ok(std::thread::spawn(|| match output {
        AudioOutput::Wav(params) => {
            export::export_to_wav(&params.path, params.spec, params.duration, sample_fn).unwrap();
        }
        AudioOutput::Direct(params) => {
            stream_loop_spinlock(params.device, params.config, sample_fn, running).unwrap()
        }
    }))
}
