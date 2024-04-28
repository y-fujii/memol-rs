// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use nih_plug::prelude::*;
use std::*;

#[derive(Params)]
struct MemolParams {}

struct MemolPlugin {
    params: sync::Arc<MemolParams>,
}

impl Default for MemolPlugin {
    fn default() -> Self {
        MemolPlugin {
            params: sync::Arc::new(MemolParams {}),
        }
    }
}

impl Plugin for MemolPlugin {
    const NAME: &'static str = "memol";
    const VENDOR: &'static str = "memol";
    const URL: &'static str = "https://mimosa-pudica.net/";
    const EMAIL: &'static str = "y-fujii@mimosa-pudica.net";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[];
    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> sync::Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        _: &mut Buffer,
        _: &mut AuxiliaryBuffers,
        _: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        ProcessStatus::Normal
    }
}

impl ClapPlugin for MemolPlugin {
    const CLAP_ID: &'static str = "net.mimosa-pudica.memol";
    const CLAP_DESCRIPTION: Option<&'static str> = None;
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect];
}

impl Vst3Plugin for MemolPlugin {
    const VST3_CLASS_ID: [u8; 16] = 0xebb785c928224555b163582e911b8c48u128.to_le_bytes();
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Instrument];
}

nih_export_clap!(MemolPlugin);
nih_export_vst3!(MemolPlugin);
