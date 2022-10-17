mod editor;

use std::{fmt::Display, ops::RangeInclusive, sync::Arc, time::Duration};

use fundsp::hacker::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use vst::prelude::*;
use wmidi::{Note, Velocity};

pub struct Parameters {
    pub modulation: AtomicFloat,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            modulation: AtomicFloat::new(1.),
        }
    }
}

#[derive(FromPrimitive, Clone, Copy)]
pub enum Parameter {
    Modulation = 0,
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Parameter::Modulation => "modulation",
            }
        )
    }
}

impl PluginParameters for Parameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match FromPrimitive::from_i32(index) {
            Some(Parameter::Modulation) => self.modulation.get(),
            _ => 0f32,
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        if let Some(Parameter::Modulation) = FromPrimitive::from_i32(index) {
            self.modulation.set(value);
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        let param: Option<Parameter> = FromPrimitive::from_i32(index);
        param
            .map(|f| f.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

#[derive(FromPrimitive, Clone, Copy)]
pub enum Tag {
    Freq = 0,
    Modulation = 1,
    NoteOn = 2,
}

struct SynthTest {
    audio: Box<dyn AudioUnit64 + Send>,
    parameters: Arc<Parameters>,
    note: Option<(Note, Velocity)>,
    enabled: bool,
    sample_rate: f32,
    time: Duration,
    editor: Option<editor::PluginEditor>,
}

impl SynthTest {
    #[inline(always)]
    fn set_tag(&mut self, tag: Tag, value: f64) {
        self.audio.set(tag as i64, value);
    }

    #[inline(always)]
    fn set_tag_with_param(&mut self, tag: Tag, param: Parameter, range: RangeInclusive<f64>) {
        let value = self.parameters.get_parameter(param as i32) as f64;
        let mapped_value = (value - range.start()) * (range.end() - range.start()) + range.start();
        self.set_tag(tag, mapped_value);
    }
}

impl Plugin for SynthTest {
    #[allow(clippy::precedence)] // Needed for chain
    fn new(_host: HostCallback) -> Self {
        let params: Arc<Parameters> = Arc::new(Default::default());
        let Parameters { modulation } = Parameters::default();

        let freq = || tag(Tag::Freq as i64, 440.);
        let modulation = || tag(Tag::Modulation as i64, modulation.get() as f64);
        let offset = || tag(Tag::NoteOn as i64, 0.);
        let env = || offset() >> envelope2(|t, offset| downarc((t - offset) * 2.));

        let audio_graph = freq()
            >> sine() * freq() * modulation() + freq()
            >> env() * sine()
            >> declick()
            >> split::<U2>();

        Self {
            audio: Box::new(audio_graph) as Box<dyn AudioUnit64 + Send>,
            parameters: params.clone(),
            note: None,
            time: Duration::default(),
            sample_rate: 44_100f32,
            enabled: false,
            editor: Some(editor::PluginEditor {
                params,
                window_handle: None,
                is_open: false,
            }),
        }
    }

    fn init(&mut self) {
        let Info {
            name,
            version,
            unique_id,
            ..
        } = self.get_info();

        let home = dirs::home_dir().unwrap().join("tmp");
        let id_string = format!("{name}-{version}-{unique_id}-log.txt");
        let log_file = std::fs::File::create(home.join(id_string)).unwrap();
        let log_config = simplelog::ConfigBuilder::new()
            .set_time_offset_to_local()
            .unwrap()
            .build();
        simplelog::WriteLogger::init(simplelog::LevelFilter::Info, log_config, log_file).ok();
        log_panics::init();
        log::info!("init");
    }

    fn get_info(&self) -> Info {
        Info {
            name: "SynthTest".into(),
            vendor: "Srynetix".into(),
            unique_id: 133713,
            category: Category::Synth,
            inputs: 0,
            outputs: 2,
            parameters: 1,
            ..Default::default()
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.parameters) as Arc<dyn PluginParameters>
    }

    fn process_events(&mut self, events: &vst::api::Events) {
        for event in events.events() {
            if let vst::event::Event::Midi(midi) = event {
                if let Ok(midi) = wmidi::MidiMessage::try_from(midi.data.as_slice()) {
                    match midi {
                        wmidi::MidiMessage::NoteOn(_channel, note, velocity) => {
                            self.set_tag(Tag::NoteOn, self.time.as_secs_f64());
                            self.note = Some((note, velocity));
                            self.enabled = true;
                        }
                        wmidi::MidiMessage::NoteOff(_channel, note, _velocity) => {
                            if let Some((current_note, ..)) = self.note {
                                if current_note == note {
                                    self.note = None;
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (_, mut outputs) = buffer.split();
        if outputs.len() == 2 {
            let (left, right) = (outputs.get_mut(0), outputs.get_mut(1));

            for (left_chunk, right_chunk) in left
                .chunks_mut(MAX_BUFFER_SIZE)
                .zip(right.chunks_mut(MAX_BUFFER_SIZE))
            {
                let mut left_buffer = [0f64; MAX_BUFFER_SIZE];
                let mut right_buffer = [0f64; MAX_BUFFER_SIZE];

                self.set_tag_with_param(Tag::Modulation, Parameter::Modulation, 0f64..=10f64);

                if let Some((note, ..)) = self.note {
                    self.set_tag(Tag::Freq, note.to_freq_f64());
                }

                if self.enabled {
                    self.time += Duration::from_secs_f32(MAX_BUFFER_SIZE as f32 / self.sample_rate);
                    self.audio.process(
                        MAX_BUFFER_SIZE,
                        &[],
                        &mut [&mut left_buffer, &mut right_buffer],
                    );
                }

                for (chunk, output) in left_chunk.iter_mut().zip(left_buffer.iter()) {
                    *chunk = *output as f32;
                }

                for (chunk, output) in right_chunk.iter_mut().zip(right_buffer.iter()) {
                    *chunk = *output as f32;
                }
            }
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
        self.time = Duration::default();
        self.audio.reset(Some(rate as f64));
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent | CanDo::ReceiveEvents => Supported::Yes,
            _ => Supported::No,
        }
    }

    fn get_editor(&mut self) -> Option<Box<dyn vst::editor::Editor>> {
        if let Some(editor) = self.editor.take() {
            Some(Box::new(editor))
        } else {
            None
        }
    }
}

vst::plugin_main!(SynthTest);
