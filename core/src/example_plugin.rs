use crate::plugin::PluginInstance;
use crate::TimedMidi;

#[derive(Copy, Clone, Debug)]
pub struct Silence;

impl PluginInstance for Silence {
    fn process(&mut self, _: &[TimedMidi<'_>], out_left: &mut [f32], out_right: &mut [f32]) {
        zero_buffer(out_left);
        zero_buffer(out_right);
    }
}

fn zero_buffer(b: &mut [f32]) {
    for o in b.iter_mut() {
        *o = 0.0;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Sine {
    sample_rate: f32,
    note: Option<wmidi::Note>,
    t: f32,
    delta_t: f32,
}

impl Sine {
    pub fn new(sample_rate: f32) -> Sine {
        Sine {
            sample_rate,
            note: None,
            t: 0.0,
            delta_t: 0.0,
        }
    }
}

impl PluginInstance for Sine {
    fn process(&mut self, midi: &[TimedMidi<'_>], out_left: &mut [f32], out_right: &mut [f32]) {
        let mut midi_iter = midi.iter().peekable();
        for (frame, output) in out_left.iter_mut().enumerate() {
            while midi_iter.peek().map(|m| m.frame <= frame).unwrap_or(false) {
                let timed_midi = midi_iter.next().unwrap();
                match timed_midi.message {
                    wmidi::MidiMessage::NoteOn(_, n, _) => {
                        self.note = Some(n);
                        let frequency = n.to_freq_f32();
                        self.delta_t = 2f32 * std::f32::consts::PI * frequency / self.sample_rate;
                    }
                    wmidi::MidiMessage::NoteOff(_, n, _) => {
                        if Some(n) == self.note {
                            self.note = None;
                            self.delta_t = 0.0;
                            self.t = 0.0;
                        }
                    }
                    _ => (),
                }
            }
            self.t += self.delta_t;
            if self.t > 2.0 * std::f32::consts::PI {
                self.t -= 2.0 * std::f32::consts::PI;
            }
            *output = self.t.sin();
        }
        out_right.copy_from_slice(out_left);
    }
}
