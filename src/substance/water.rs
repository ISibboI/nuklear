use uom::num_traits::Zero;
use uom::si::available_energy::joule_per_gram;
use uom::si::f64::AvailableEnergy;
use uom::si::ratio::ratio;
use uom::si::{
    f64::{SpecificHeatCapacity, Mass, ThermodynamicTemperature, Volume},
    specific_heat_capacity::joule_per_kilogram_kelvin,
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

    /// The heat capacity of water.
    /// We treat it as the same over all temperatures and phases to avoid creating or losing energy due to moving boiling point.
    pub fn heat_capacity() -> SpecificHeatCapacity {
        SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(4180.0)
    }

    pub fn evaporization_energy() -> AvailableEnergy {
        AvailableEnergy::new::<joule_per_gram>(2230.0)
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
    // boiling point at low pressure https://www.myengineeringtools.com/Data_Diagrams/Water_Boiling_Point_Vs_Pressure.html
    // boiling point at high pressure https://www.engineeringtoolbox.com/water-vapor-saturation-pressure-d_599.html

    use lazy_static::lazy_static;
    use uom::si::{
        f64::{MassDensity, Pressure, ThermodynamicTemperature},
        mass_density::gram_per_cubic_centimeter,
        pressure::millibar,
        thermodynamic_temperature::degree_celsius,
    };

    use crate::interpolation_table::{LimitBehaviour, LinearInterpolationTable};

    lazy_static! {
        /// Celsius -> g/cm^3
        /// High temperatures (above 100) roughly at boiling pressure
        static ref DENSITY_BY_TEMPERATURE: LinearInterpolationTable =
            LinearInterpolationTable::new(LimitBehaviour::Clamp, vec![
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
                (120.0, 0.9435),
                (140.0, 0.9262),
                (160.0, 0.9075),
                (180.0, 0.887),
                (200.0, 0.8647),
                (225.0, 0.8338),
                (250.0, 0.799),
                (275.0, 0.7591),
                (300.0, 0.7122),
                (350.0, 0.581),
                (373.0, 0.4434),
                (373.946, 0.3888),
                (400.0, 0.1222),
                (450.0, 0.09043),
                (500.0, 0.0765),
                (600.0, 0.06145),
                (700.0, 0.0526),
                (800.0, 0.04645),
                (900.0, 0.0418),
                (1000.0, 0.0381),
                (2000.0, 0.02085),
            ]);

        /// mbar -> Celsius
        /// above the critical point, we treat it as liquid
        static ref BOILING_POINT_BY_PRESSURE_RAW: Vec<(f64, f64)> = vec![
            (0.003, -68.0),
            (0.017, -57.0),
            (0.03, -51.0),
            (0.07, -46.0),
            (0.13, -40.0),
            (0.17, -37.0),
            (0.34, -31.0),
            (0.4, -29.0),
            (0.67, -24.0),
            (1.33, -17.0),
            (1.69, -14.0),
            (3.39, -6.0),
            (6.1, 0.0),
            (10.16, 7.0),
            (13.55, 12.0),
            (16.93, 15.0),
            (20.32, 18.0),
            (23.71, 21.0),
            (27.09, 22.0),
            (30.48, 24.0),
            (33.86, 27.0),
            (42.33, 30.0),
            (73.48, 40.0),
            (123.3, 50.0),
            (133.3, 52.0),
            (199.1, 60.0),
            (266.6, 67.0),
            (311.5, 70.0),
            (473.4, 80.0),
            (666.6, 89.0),
            (700.6, 90.0),
            (846.6, 96.0),
            (1014.2, 100.3),
            (1433.8, 110.0),
            (1986.7, 120.0),
            (2702.8, 130.0),
            (3615.4, 140.0),
            (4761.6, 150.0),
            (6182.3, 160.0),
            (10028.0, 180.0),
            (15549.0, 200.0),
            (23196.0, 220.0),
            (33469.0, 240.0),
            (46923.0, 260.0),
            (64166.0, 280.0),
            (85879.0, 300.0),
            (112840.0, 320.0),
            (146010.0, 340.0),
            (186660.0, 360.0),
            (210440.0, 370.0),
            (210441.0, 1e50),
        ];

        static ref BOILING_POINT_BY_PRESSURE: LinearInterpolationTable = LinearInterpolationTable::new(LimitBehaviour::Clamp, BOILING_POINT_BY_PRESSURE_RAW.clone());

        static ref SATURATION_PRESSURE_BY_TEMPERATURE: LinearInterpolationTable = LinearInterpolationTable::new(LimitBehaviour::Clamp, BOILING_POINT_BY_PRESSURE_RAW.iter().copied().map(|(pressure, temperature)| (temperature, pressure)).collect());
    }

    pub fn density_by_temperature(temperature: ThermodynamicTemperature) -> MassDensity {
        let temperature = temperature.get::<degree_celsius>();
        let density = DENSITY_BY_TEMPERATURE.get(temperature);
        MassDensity::new::<gram_per_cubic_centimeter>(density)
    }

    pub fn boiling_point_by_pressure(pressure: Pressure) -> ThermodynamicTemperature {
        let pressure = pressure.get::<millibar>();
        let temperature = BOILING_POINT_BY_PRESSURE.get(pressure);
        ThermodynamicTemperature::new::<degree_celsius>(temperature)
    }

    pub fn saturation_pressure_by_temperature(temperature: ThermodynamicTemperature) -> Pressure {
        let temperature = temperature.get::<degree_celsius>();
        let pressure = SATURATION_PRESSURE_BY_TEMPERATURE.get(temperature);
        Pressure::new::<millibar>(pressure)
    }
}
