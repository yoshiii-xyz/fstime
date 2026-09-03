#![no_main]

use fstime::{explain_comparison, explain_profile, ProfileComparison, ProfileReport};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    if let Ok(report) = serde_json::from_str::<ProfileReport>(&input) {
        let _ = explain_profile(&report);
    }
    if let Ok(report) = serde_json::from_str::<ProfileComparison>(&input) {
        let _ = explain_comparison(&report);
    }
});
