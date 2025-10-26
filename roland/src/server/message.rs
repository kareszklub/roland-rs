use serde::{Deserialize, Serialize};

/// This is the message a client can send to control Roland
#[derive(Debug, Deserialize)]
pub enum ClientMessage {
    /// frequency (Hz)
    Buzzer(u16),
    /// RGB color (0 to 255)
    LED((u8, u8, u8)),
    /// rotation in degrees (0 is the midpoint, -90 to 90)
    Servo(i8),
    /// Motor duty cycle (-1 to 1)
    Motor((f32, f32)),
    FollowLine,
}

/// This is the message Roland can send to a client
#[derive(Debug, Serialize)]
pub enum ServerMessage {
    /// This is self-explanatory, Roland is NOT a chill dude
    GoFuckYourself,
}
