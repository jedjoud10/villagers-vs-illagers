use crate::wasm4::*;

pub enum Noise {
    Ting,
    SixSeven,
    TungTungTungSahour,
}

pub unsafe fn play_me_some_tones______boy(noise: Noise) {
    match noise {
        Noise::Ting => tone(300, 3, 5, TONE_MODE1),
        Noise::SixSeven => tone(100, 3, 5, TONE_MODE1),
        Noise::TungTungTungSahour => tone(600, 3, 5, TONE_MODE1),
    }
}