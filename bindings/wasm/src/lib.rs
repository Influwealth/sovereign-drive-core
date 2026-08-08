use sovereigndrive_engine::SovereignDriveEngine;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SovereignDriveWasm {
    engine: SovereignDriveEngine,
}

#[wasm_bindgen]
impl SovereignDriveWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { engine: SovereignDriveEngine::default() }
    }

    pub fn ingest(&self, data: &[u8]) -> Result<String, JsValue> {
        self.engine
            .ingest_bytes(data)
            .map(|result| result.hash)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn read(&self, hash: &str) -> Result<Vec<u8>, JsValue> {
        self.engine
            .read_bytes(hash)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }
}

impl Default for SovereignDriveWasm {
    fn default() -> Self { Self::new() }
}
