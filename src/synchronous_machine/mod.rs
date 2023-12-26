use std::{f64::consts::TAU, marker::PhantomData};

use uom::si::{
    angle::revolution,
    angular_velocity::revolution_per_second,
    f64::{
        Angle, AngularVelocity, ElectricPotential, ElectricalResistance, MagneticFlux,
        MomentOfInertia, Power, Time,
    },
    frequency::hertz,
    ratio::ratio,
    Quantity,
};

pub struct SynchronousMachine {
    moment_of_inertia: MomentOfInertia,
    angular_position: Angle,
    angular_velocity: AngularVelocity,
    power_rating: Power,
    flux_linkage: MagneticFlux,
    /// The amount of power required to magnetise the rotor.
    power_requirement: Power,
}

impl SynchronousMachine {
    pub fn new(
        moment_of_inertia: MomentOfInertia,
        angular_position: Angle,
        angular_velocity: AngularVelocity,
        power_rating: Power,
        flux_linkage: MagneticFlux,
        power_requirement: Power,
    ) -> Self {
        Self {
            moment_of_inertia,
            angular_position,
            angular_velocity,
            power_rating,
            flux_linkage,
            power_requirement,
        }
    }

    pub fn moment_of_inertia(&self) -> MomentOfInertia {
        self.moment_of_inertia
    }

    pub fn angular_position(&self) -> Angle {
        self.angular_position
    }

    pub fn angular_velocity(&self) -> AngularVelocity {
        self.angular_velocity
    }

    pub fn power_rating(&self) -> Power {
        self.power_rating
    }

    pub fn flux_linkage(&self) -> MagneticFlux {
        self.flux_linkage
    }

    pub fn power_requirement(&self) -> Power {
        self.power_requirement
    }

    pub fn peak_internal_voltage(&self) -> ElectricPotential {
        self.angular_velocity * self.flux_linkage
    }

    pub fn apply_mechanical_electrical_power(
        &mut self,
        mechanical_acceleration_power: Power,
        electrical_deceleration_power: Power,
        time: Time,
    ) {
        let angular_acceleration_power =
            mechanical_acceleration_power - electrical_deceleration_power;
        let angular_acceleration =
            angular_acceleration_power / (self.moment_of_inertia * self.angular_velocity);
        let angular_velocity_increment = angular_acceleration * time;
        self.angular_velocity += AngularVelocity::new::<revolution_per_second>(
            angular_velocity_increment.get::<hertz>(),
        );
    }

    pub fn rotate(&mut self, time: Time) {
        let angular_increment = self.angular_velocity * time;
        self.angular_position += Angle::new::<revolution>(angular_increment.get::<ratio>());
        self.angular_position %= Angle::FULL_TURN;
    }

    /// Sum of electrical power of each phase.
    pub fn sum_of_phase_powers(
        &self,
        power_grid_voltage: ElectricPotential,
        power_grid_angular_position: Angle,
        power_grid_resistance: ElectricalResistance,
    ) -> Power {
        PHASE_OFFSETS
            .iter()
            .copied()
            .map(|phase_offset| {
                let angular_position = self.angular_position + phase_offset;
                debug_assert!(angular_position >= -Angle::FULL_TURN - Angle::FULL_TURN);
                debug_assert!(angular_position < Angle::FULL_TURN + Angle::FULL_TURN);

                let power_grid_phase_voltage =
                    power_grid_voltage * (power_grid_angular_position + phase_offset).sin();
                let phase_internal_voltage = self.peak_internal_voltage() * angular_position.sin();
                let voltage_lead = phase_internal_voltage - power_grid_phase_voltage;
                let current = voltage_lead / power_grid_resistance;
                // println!("{phase_offset:?}: {current:?} * {phase_internal_voltage:?}");
                current * phase_internal_voltage
            })
            .sum::<Power>()
    }
}

const PHASE_OFFSETS: &[Angle] = &[
    Quantity {
        dimension: PhantomData,
        units: PhantomData,
        value: 0.0,
    },
    Quantity {
        dimension: PhantomData,
        units: PhantomData,
        value: TAU / 3.0,
    },
    Quantity {
        dimension: PhantomData,
        units: PhantomData,
        value: TAU * 2.0 / 3.0,
    },
];

#[cfg(test)]
mod tests {
    use std::f64::consts::TAU;

    use uom::{
        fmt::DisplayStyle,
        si::{
            angle::{degree, radian},
            angular_velocity::revolution_per_second,
            electric_potential::volt,
            electrical_resistance::ohm,
            f64::{
                Angle, AngularVelocity, ElectricPotential, ElectricalResistance, MagneticFlux,
                MomentOfInertia, Power,
            },
            magnetic_flux::weber,
            moment_of_inertia::kilogram_square_meter,
            power::watt,
        },
        ConstZero,
    };

    use crate::synchronous_machine::PHASE_OFFSETS;

    use super::SynchronousMachine;

    #[test]
    fn electric_current() {
        assert!((PHASE_OFFSETS[0] - Angle::ZERO).get::<degree>().abs() < 1e-10);
        assert!(
            (PHASE_OFFSETS[1] - Angle::new::<degree>(120.0))
                .get::<degree>()
                .abs()
                < 1e-10
        );
        assert!(
            (PHASE_OFFSETS[2] - Angle::new::<degree>(240.0))
                .get::<degree>()
                .abs()
                < 1e-10
        );

        let mut generator = SynchronousMachine::new(
            MomentOfInertia::new::<kilogram_square_meter>(1.0),
            Angle::new::<radian>(0.0),
            AngularVelocity::new::<revolution_per_second>(1.0),
            Power::new::<watt>(1.0),
            MagneticFlux::new::<weber>(1.0 / TAU),
            Power::ZERO,
        );

        let expected_generator_voltage = ElectricPotential::new::<volt>(1.0);
        let generator_voltage = generator.peak_internal_voltage();
        let power_grid_voltage = generator_voltage;
        assert!(
            (generator_voltage - expected_generator_voltage)
                .abs()
                .get::<volt>()
                < 1e-10,
            "expected: {}; actual: {}",
            expected_generator_voltage.into_format_args(volt, DisplayStyle::Abbreviation),
            generator_voltage.into_format_args(volt, DisplayStyle::Abbreviation)
        );

        for angle in 0..12 {
            let angle = angle * 30;
            generator.angular_position = Angle::new::<degree>(f64::from(angle));
            let power = generator.sum_of_phase_powers(
                power_grid_voltage,
                Angle::new::<radian>(0.0),
                ElectricalResistance::new::<ohm>(1.0),
            );

            if angle == 0 {
                assert!(power.get::<watt>().abs() < 1e-10);
            }
            println!(
                "angle: {:.0}; power: {:.2}",
                generator
                    .angular_position
                    .into_format_args(degree, DisplayStyle::Abbreviation),
                power.into_format_args(watt, DisplayStyle::Abbreviation)
            )
        }
    }
}
