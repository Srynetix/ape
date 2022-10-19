use std::sync::{
    atomic::{AtomicBool, AtomicU16, Ordering},
    Arc,
};

use ape_core::{color_eyre::eyre, dsp::build_dsp_chain_pitch, start_stream_thread, AudioOutput};

use eframe::egui;
use egui::Slider;

#[derive(Default)]
struct MyApp {
    sound_enabled: Arc<AtomicBool>,
    pitch: Arc<AtomicU16>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ape");

            let mut enabled = self.sound_enabled.load(Ordering::Relaxed);
            if ui.checkbox(&mut enabled, "Enable sound").changed() {
                self.sound_enabled.store(enabled, Ordering::Relaxed);
            }

            let mut pitch = self.pitch.load(Ordering::Relaxed) as f64;
            if ui
                .add(
                    Slider::new(&mut pitch, 220.0..=220.0 * 2.0)
                        .step_by(1.0)
                        .text("Pitch"),
                )
                .changed()
            {
                self.pitch.store(pitch as u16, Ordering::Relaxed);
            }
        });
    }
}

fn main() -> eyre::Result<()> {
    let audio_output = AudioOutput::new_direct()?;
    let sound_running = Arc::new(AtomicBool::new(true));
    let sound_enabled = Arc::new(AtomicBool::new(false));
    let pitch = Arc::new(AtomicU16::new(220));
    let sample_rate = audio_output.sample_rate();

    let app = Box::new(MyApp {
        sound_enabled: sound_enabled.clone(),
        pitch: pitch.clone(),
    });

    let mut last_pitch = pitch.load(Ordering::Relaxed) as f64;
    let mut chain = build_dsp_chain_pitch(last_pitch, sample_rate);
    let sample_fn = move || {
        if sound_enabled.load(Ordering::Relaxed) {
            let this_pitch = pitch.load(Ordering::Relaxed) as f64;
            if last_pitch != this_pitch {
                chain = build_dsp_chain_pitch(this_pitch, sample_rate);
                last_pitch = this_pitch;
            }

            let v = chain.get_stereo();
            vec![v.0 as f32, v.1 as f32]
        } else {
            vec![0.0, 0.0]
        }
    };

    let handle = start_stream_thread(audio_output, sample_fn, sound_running.clone())?;

    let options = eframe::NativeOptions::default();
    eframe::run_native("ape", options, Box::new(|_cc| app));

    sound_running.store(false, Ordering::Relaxed);
    handle.join().expect("could not join thread");

    Ok(())
}
