use sovereigndrive_engine::SovereignDriveEngine;

#[test]
fn ingest_round_trip_smoke() {
    let engine = SovereignDriveEngine::default();
    let data = b"hello world";
    let result = engine.ingest_bytes(data).unwrap();
    assert_eq!(engine.read_bytes(&result.hash).unwrap(), data);
}
