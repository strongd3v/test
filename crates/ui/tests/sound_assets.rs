//! The rendered sound cues ship from one render; every platform copy of the
//! same cue must stay byte-identical to the shared asset.

use std::fs;
use std::path::PathBuf;

fn repo_root(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(rel)
}

fn asset(rel: &str) -> Vec<u8> {
    fs::read(repo_root(rel)).unwrap_or_else(|err| panic!("sound asset {rel} is missing: {err}"))
}

#[test]
fn platform_copies_of_voice_cues_are_byte_identical() {
    let ui_start = asset("crates/ui/assets/sounds/voice-start.wav");
    let ui_end = asset("crates/ui/assets/sounds/voice-end.wav");
    let ios_start = asset("apps/ios/Space/Voice/Sounds/voice-start.wav");
    let ios_end = asset("apps/ios/Space/Voice/Sounds/voice-end.wav");
    let android_start = asset("apps/android/app/src/main/res/raw/voice_start.wav");
    let android_end = asset("apps/android/app/src/main/res/raw/voice_end.wav");

    assert!(ui_start.len() > 1000);
    assert!(ui_end.len() > 1000);
    assert_eq!(&ui_start[..4], b"RIFF");
    assert_eq!(&ui_end[..4], b"RIFF");

    assert_eq!(ui_start, ios_start, "iOS voice-start copy diverged");
    assert_eq!(ui_start, android_start, "Android voice_start copy diverged");
    assert_eq!(ui_end, ios_end, "iOS voice-end copy diverged");
    assert_eq!(ui_end, android_end, "Android voice_end copy diverged");
}
