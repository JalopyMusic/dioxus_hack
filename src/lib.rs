use dioxus::prelude::*;
use dioxus_desktop::tao::platform::macos::WindowBuilderExtMacOS;
use nih_plug::prelude::*;
use std::sync::Arc;

fn root(cx: Scope) -> Element {
    render!(
        div {
            style: "width: 300px; height: 300px; background-color: pink; color: blue;",
            "Howdy nih_plug world!!!"
        }
    )
}

struct ParentWindowWrapper {
    parent_window: *mut ::std::os::raw::c_void,
}

unsafe impl Send for ParentWindowWrapper {}

unsafe impl Sync for ParentWindowWrapper {}

struct DioxusEditorHandle;

struct DioxusEditor;

impl Editor for DioxusEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        _context: Arc<dyn GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        let parent_window_wrapper = Arc::new(ParentWindowWrapper {
            parent_window: match parent {
                ParentWindowHandle::X11Window(_) => todo!(),
                ParentWindowHandle::AppKitNsView(p) => p,
                ParentWindowHandle::Win32Hwnd(p) => p,
            },
        });

        std::thread::spawn(move || {
            let size = dioxus_desktop::LogicalSize {
                width: 300,
                height: 300,
            };

            let window = dioxus_desktop::WindowBuilder::default()
                .with_parent_window(parent_window_wrapper.parent_window)
                .with_inner_size(size)
                .with_min_inner_size(size)
                .with_max_inner_size(size)
                .with_closable(true)
                .with_focused(true)
                .with_maximizable(false)
                .with_maximized(false)
                .with_minimizable(false)
                .with_resizable(false)
                .with_title("Dioxus Hack");

            let config = dioxus_desktop::Config::default()
                .with_background_color((255, 255, 255, 255))
                .with_close_behaviour(dioxus_desktop::WindowCloseBehaviour::LastWindowHides)
                .with_window(window);

            dioxus_desktop::launch_cfg(root, config);
        });

        Box::new(DioxusEditorHandle {})
    }

    fn size(&self) -> (u32, u32) {
        (300, 300)
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        todo!()
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {
        todo!()
    }

    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {
        todo!()
    }

    fn param_values_changed(&self) {
        todo!()
    }
}

#[derive(Params)]
struct MyPluginParams {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for MyPluginParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}

struct MyPlugin {
    params: Arc<MyPluginParams>,
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(MyPluginParams::default()),
        }
    }
}

impl Plugin for MyPlugin {
    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        Some(Box::new(DioxusEditor {}))
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn reset(&mut self) {}

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();

            for sample in channel_samples {
                *sample *= gain;
            }
        }

        ProcessStatus::Normal
    }

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    type SysExMessage = ();
    type BackgroundTask = ();
    const NAME: &'static str = "Dioxus Hack";
    const VENDOR: &'static str = "Brian Edwards";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "brian.edwards@jalopymusic.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];
}

impl ClapPlugin for MyPlugin {
    const CLAP_ID: &'static str = "com.jalopymusic.dioxus-hack";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Hack in a Dioxus editor");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for MyPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"DioxusHackJalopy";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(MyPlugin);
nih_export_vst3!(MyPlugin);
