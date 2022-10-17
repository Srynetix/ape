use color_eyre::eyre;

use ape_core::{process_stream, AudioOutput};
use rlua::Lua;

fn bytebeats_to_f32(v: u32) -> f32 {
    (v & 255) as f32 / 127.0 - 1.0
}

pub fn run_bytebeats_synth(output: AudioOutput, formula: String) -> eyre::Result<()> {
    let resample_ratio = 8_000.0 / output.sample_rate() as f32;
    let lua = Lua::new();

    lua.context(|ctx| {
        let globs = ctx.globals();
        globs.set("t", 0).unwrap();
    });

    let mut count = 0.0;
    let sample_fn = move || {
        let t = count as u32;

        let formula = lua.context(|ctx| {
            let globs = ctx.globals();
            globs
                .set("t", t)
                .expect("could not update the 't' variable");
            ctx.load(&formula)
                .eval()
                .expect("could not evaluate the formula")
        });

        let f = bytebeats_to_f32(formula);
        count += resample_ratio;
        vec![f, f]
    };

    process_stream(output, sample_fn)?;

    Ok(())
}
