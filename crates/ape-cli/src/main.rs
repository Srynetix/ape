use std::path::PathBuf;

use ape_bytebeats::run_bytebeats_synth;
use ape_core::{
    color_eyre::{self, eyre},
    dsp::{build_dsp_chain, sample_noise_fn},
    process_stream, AudioOutput, WavOutput,
};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Export to .wav
    #[arg(short, long)]
    wav: Option<PathBuf>,

    /// Sample rate
    #[arg(short, long)]
    sample_rate: Option<u32>,

    /// Duration
    #[arg(short, long)]
    duration: Option<usize>,

    /// Command
    #[command(subcommand)]
    cmd: SubCmd,
}

#[derive(Subcommand, Debug)]
enum SubCmd {
    Bytebeats(BytebeatsCmd),
    Noise,
    Dsp,
}

#[derive(Parser, Debug)]
struct BytebeatsCmd {
    /// Formula
    formula: String,
}

fn build_audio_output(args: &Args) -> eyre::Result<AudioOutput> {
    if let Some(path) = &args.wav {
        Ok(AudioOutput::Wav(WavOutput {
            path: path.into(),
            duration: args.duration.unwrap_or(3),
            spec: ape_core::hound::WavSpec {
                bits_per_sample: 16,
                channels: 2,
                sample_format: ape_core::hound::SampleFormat::Int,
                sample_rate: args.sample_rate.unwrap_or(44_100),
            },
        }))
    } else {
        AudioOutput::new_direct()
    }
}

fn run_dsp_synth(output: AudioOutput) -> eyre::Result<()> {
    let mut chain = build_dsp_chain(output.sample_rate());
    let sample_fn = move || {
        let v = chain.get_stereo();
        vec![v.0 as f32, v.1 as f32]
    };

    // let sample_fn = sample_noise_fn;
    process_stream(output, sample_fn)
}

fn setup_logging() -> eyre::Result<()> {
    tracing_subscriber::fmt().init();

    Ok(())
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    setup_logging()?;

    let args = Args::parse();
    let output = build_audio_output(&args)?;

    match args.cmd {
        SubCmd::Bytebeats(bb) => {
            run_bytebeats_synth(output, bb.formula)?;
        }
        SubCmd::Dsp => {
            run_dsp_synth(output)?;
        }
        SubCmd::Noise => {
            process_stream(output, sample_noise_fn)?;
        }
    }

    Ok(())
}
