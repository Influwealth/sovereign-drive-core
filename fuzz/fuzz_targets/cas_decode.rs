#![no_main]

use libfuzzer_sys::fuzz_target;
use sovereigndrive_engine::cas::validate_encoded_object;

fuzz_target!(|data: &[u8]| {
    let _ = validate_encoded_object(&"00".repeat(32), data);
});
