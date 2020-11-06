pub use uom::si::{
    acceleration::foot_per_second_squared,
    angle::degree,
    length::foot,
    velocity::{foot_per_minute, knot},
};

use uom::si::f64::*;

pub struct Attitude {
    ///
    pitch: Angle,
    roll: Angle,
}

/// Structure describing the current state of an Aicraft
pub struct AircraftState {
    /// Height above sealevel in foot
    pub altitude_sealevel: Length,

    /// Height above current terrain in foot
    pub altitude_ground: Length,

    /// Rate of descent
    pub climb_rate: Velocity,

    /// Geographic Latitude, specifying the north-south position
    pub position_lat: Angle,

    /// Geographic Longitude, specifying the east-west position
    pub position_lon: Angle,

    /// Angle in degrees (clockwise) between north and the direction to the
    /// destination or nav aid
    //pub bearing: degree,

    /// Angle in degrees (clockwise) between north and the direction where the
    /// aircrafts nose is pointing
    pub heading: Angle,

    /// Estimated aicraft speed
    pub speed: Velocity,

    /// Attitude of the aircraft including pitch & roll
    pub attitude: Attitude,
}

#[derive(Clone)]
pub struct TAWSConfig {
    max_climbrate: Velocity,
    max_climbrate_change: Acceleration,
}

impl Default for TAWSConfig {
    fn default() -> Self {
        Self {
            max_climbrate: Velocity::new::<foot_per_minute>(700.0),
            max_climbrate_change: Acceleration::new::<foot_per_second_squared>(100.0),
        }
    }
}