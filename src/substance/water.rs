use uom::num_traits::Zero;
use uom::si::ratio::ratio;
use uom::si::{
    f64::{HeatCapacity, Mass, ThermodynamicTemperature, Volume},
    heat_capacity::joule_per_kelvin,
    mass::kilogram,
    temperature_interval,
    thermodynamic_temperature::kelvin,
};

use self::constants::density_by_temperature;

#[derive(Debug, Clone, Copy)]
pub struct Water {
    mass: Mass,
    temperature: ThermodynamicTemperature,
}

impl Water {
    pub fn new(mass: Mass, temperature: ThermodynamicTemperature) -> Self {
        Self { mass, temperature }
    }

    pub fn zero() -> Self {
        Self {
            mass: Mass::new::<kilogram>(0.0),
            temperature: ThermodynamicTemperature::new::<kelvin>(0.0),
        }
    }

    pub fn heat_capacity() -> HeatCapacity {
        HeatCapacity::new::<joule_per_kelvin>(4.2)
    }

    pub fn volume(&self) -> Volume {
        let density = density_by_temperature(self.temperature);
        self.mass / density
    }
}

impl std::ops::Add for Water {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mass = self.mass + rhs.mass;
        let temperature = if mass > Mass::zero() {
            ThermodynamicTemperature::new::<kelvin>(
                ((self.mass * self.temperature + rhs.mass * rhs.temperature) / mass)
                    .get::<temperature_interval::kelvin>(),
            )
        } else {
            ThermodynamicTemperature::new::<kelvin>(0.0)
        };

        Self { mass, temperature }
    }
}

impl std::ops::Sub<Mass> for Water {
    type Output = Self;

    fn sub(self, rhs: Mass) -> Self::Output {
        assert!(rhs <= self.mass);
        Self {
            mass: self.mass - rhs,
            ..self
        }
    }
}

impl std::ops::Sub<Volume> for Water {
    type Output = Self;

    fn sub(self, rhs: Volume) -> Self::Output {
        let volume = self.volume();
        assert!(rhs <= volume);
        let fraction = rhs / volume;
        let fraction = fraction.get::<ratio>();
        assert!((0.0..=1.0).contains(&fraction));
        let mass = self.mass * fraction;

        self - mass
    }
}

mod constants {
    use lazy_static::lazy_static;
    use uom::si::{
        f64::{MassDensity, ThermodynamicTemperature},
        mass_density::gram_per_cubic_centimeter,
        thermodynamic_temperature::degree_celsius,
    };

    use crate::interpolation_table::LinearInterpolationTable;

    lazy_static! {
        /// Celsius -> g/cm^3
        static ref DENSITY_BY_TEMPERATURE: LinearInterpolationTable =
            LinearInterpolationTable::new(vec![
                (-30.0, 0.983854),
                (-20.0, 0.993547),
                (-10.0, 0.998117),
                (0.0, 0.9998395),
                (3.984, 0.999972),
                (4.0, 0.999972),
                (5.0, 0.99996),
                (10.0, 0.9997026),
                (15.0, 0.9991026),
                (20.0, 0.9982071),
                (22.0, 0.9977735),
                (25.0, 0.9970479),
                (30.0, 0.9956502),
                (35.0, 0.99403),
                (40.0, 0.99221),
                (45.0, 0.99022),
                (50.0, 0.98804),
                (55.0, 0.98570),
                (60.0, 0.98321),
                (65.0, 0.98056),
                (70.0, 0.97778),
                (75.0, 0.97486),
                (80.0, 0.97180),
                (85.0, 0.96862),
                (90.0, 0.96531),
                (95.0, 0.96189),
                (100.0, 0.95835),
            ]);
    }

    pub fn density_by_temperature(temperature: ThermodynamicTemperature) -> MassDensity {
        let temperature = temperature.get::<degree_celsius>();
        let density = DENSITY_BY_TEMPERATURE.get(temperature);
        MassDensity::new::<gram_per_cubic_centimeter>(density)
    }
}
