use tokio::sync::watch;

pub type UltraData = Option<u16>;
pub type TrackData = [bool; 4];

/// wrapper for all sensor state
#[derive(Clone)]
pub struct Sensors {
    pub ultra_sensor: watch::Sender<UltraData>,
    pub track_sensor: watch::Sender<TrackData>,
}
