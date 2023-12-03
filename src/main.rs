use substance::water::Water;
use uom::si::{
    f64::{Mass, ThermodynamicTemperature, Volume},
    mass::kilogram,
    thermodynamic_temperature::{degree_celsius, kelvin},
    volume::cubic_decimeter,
};

pub mod container;
pub mod interpolation_table;
pub mod substance;

fn main() {
    let water_celsius = Water::new(
        Mass::new::<kilogram>(1.0),
        ThermodynamicTemperature::new::<degree_celsius>(80.0),
    );
    dbg!(water_celsius);

    let water_kelvin = Water::new(
        Mass::new::<kilogram>(1.0),
        ThermodynamicTemperature::new::<kelvin>(300.0),
    );
    dbg!(water_kelvin);

    let water = water_celsius + water_kelvin;
    dbg!(water);

    dbg!(water - Volume::new::<cubic_decimeter>(1.0));
}
