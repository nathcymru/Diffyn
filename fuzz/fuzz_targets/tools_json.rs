#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| diffyn_fuzz::check_json(data));
