use proptest::prelude::*;
use sovereigndrive_engine::SovereignDriveEngine;
use tempfile::tempdir;

proptest! {
    #[test]
    fn ingest_read_is_identity(data in prop::collection::vec(any::<u8>(), 0..8192)) {
        let engine = SovereignDriveEngine::default();
        let result = engine.ingest_bytes(&data).unwrap();
        prop_assert_eq!(engine.read_bytes(&result.hash).unwrap(), data);
    }

    #[test]
    fn persistent_ingest_survives_reopen(data in prop::collection::vec(any::<u8>(), 0..4096)) {
        let dir = tempdir().unwrap();
        let engine = SovereignDriveEngine::persistent(dir.path()).unwrap();
        let result = engine.ingest_bytes(&data).unwrap();
        drop(engine);
        let reopened = SovereignDriveEngine::persistent(dir.path()).unwrap();
        prop_assert_eq!(reopened.read_bytes(&result.hash).unwrap(), data);
    }
}
