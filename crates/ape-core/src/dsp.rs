use fundsp::hacker::*;
use rand::Rng;

pub fn sample_noise_fn() -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let v = rng.gen_range(-1.0..1.0);
    vec![v, v]
}

pub fn build_dsp_chain(sample_rate: u32) -> Box<dyn AudioUnit64> {
    let c = lfo(|t| {
        let pitch = 440.0;
        let duty = lerp11(0.01, 0.99, sin_hz(0.05 * 4.0, t));
        (pitch, duty)
    }) >> pulse();

    let mut c = c >> split::<U2>();
    c.reset(Some(sample_rate as f64));

    Box::new(c)
}
