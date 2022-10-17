use std::path::Path;

use color_eyre::eyre;

pub fn export_to_wav(
    path: &Path,
    spec: hound::WavSpec,
    duration: usize,
    mut sample_fn: impl FnMut() -> Vec<f32>,
) -> eyre::Result<()> {
    let mut writer = hound::WavWriter::create(path, spec)?;
    for _ in 0..(spec.sample_rate * duration as u32) {
        let samples = sample_fn();
        for sample in samples {
            let value = match spec.bits_per_sample {
                16 => (sample * 32768.0) as i16,
                8 => (sample * 127.0) as i16,
                _ => panic!("Oops"),
            };
            writer.write_sample(value)?;
        }
    }

    Ok(())
}
