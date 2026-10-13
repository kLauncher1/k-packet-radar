use k_packet_radar::{ObjectCategory, RadarGrid};

#[test]
fn test_radar_grid_normalization() {
    let mut radar = RadarGrid::new(1000, 1000);

    // Center point (500, 500) -> grid (127, 127)
    radar.update_object(1, 0, ObjectCategory::CombatUnit, 500, 500, 10);

    let objects = radar.get_objects_for_player(0);
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].grid_x, 127);
    assert_eq!(objects[0].grid_y, 127);
}

#[test]
fn test_radar_ascii_rendering() {
    let mut radar = RadarGrid::new(100, 100);

    radar.update_object(1, 0, ObjectCategory::Worker, 0, 0, 1);
    radar.update_object(2, 1, ObjectCategory::Building, 99, 99, 1);

    let ascii = radar.render_ascii_minimap(10, 10);
    assert!(ascii.contains('w'));
    assert!(ascii.contains('B'));
}
