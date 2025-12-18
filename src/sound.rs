use crate::wasm4::*;

pub enum Noise {
    Ting,
    SixSeven,
    TungTungTungSahour,
    KaboomNoise,
    Burn,
}

pub unsafe fn play_me_some_tones______boy(noise: Noise) {
    match noise {
        Noise::Ting => tone(300, 3, 5, TONE_MODE1),
        Noise::SixSeven => tone(100, 3, 5, TONE_MODE1),
        Noise::TungTungTungSahour => tone(600, 3, 5, TONE_MODE1),
        Noise::KaboomNoise => {
            tone(300, 60, 15, TONE_NOISE);
            tone(300, 30, 8, TONE_TRIANGLE);
            tone(300, 25, 9, TONE_TRIANGLE);
        }
        Noise::Burn => {
            tone(300, 20, 5, TONE_NOISE);
        }
    }
}