use std::{f64::consts::TAU, fmt::Display, marker::PhantomData};

use uom::{
    fmt::DisplayStyle,
    si::{
        angle::revolution,
        angular_velocity::radian_per_second,
        f64::{
            Angle, AngularVelocity, ElectricPotential, Energy, MagneticFlux, MomentOfInertia,
            Power, Time,
        },
        power::watt,
        ratio::ratio,
        time::second,
        Quantity,
    },
    ConstZero,
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

    pub fn internal_voltages_per_phase(&self) -> [ElectricPotential; 3] {
        [
            self.peak_internal_voltage() * (self.angular_position + PHASE_OFFSETS[0]).sin(),
            self.peak_internal_voltage() * (self.angular_position + PHASE_OFFSETS[1]).sin(),
            self.peak_internal_voltage() * (self.angular_position + PHASE_OFFSETS[2]).sin(),
        ]
    }

    pub fn signed_angular_kinetic_energy(&self) -> Energy {
        self.moment_of_inertia * self.angular_velocity * self.angular_velocity / 2.0
            * self.angular_velocity.value.signum()
    }

    fn apply_mechanical_electrical_power(
        &mut self,
        mechanical_acceleration_power: Power,
        electrical_deceleration_power: Power,
        delta_time: Time,
    ) {
        //dbg!(self.angular_velocity);
        let angular_acceleration_power =
            mechanical_acceleration_power - electrical_deceleration_power;
        //dbg!(angular_acceleration_power);
        let signed_angular_kinetic_energy = self.signed_angular_kinetic_energy();
        //dbg!(signed_angular_kinetic_energy);
        let kinetic_energy_increment = angular_acceleration_power * delta_time;
        //dbg!(kinetic_energy_increment);
        let signed_angular_kinetic_energy =
            signed_angular_kinetic_energy + kinetic_energy_increment;
        //dbg!(signed_angular_kinetic_energy);
        self.angular_velocity = AngularVelocity::new::<radian_per_second>(
            (signed_angular_kinetic_energy.value.signum()
                * (signed_angular_kinetic_energy.abs() * 2.0 / self.moment_of_inertia).sqrt())
            .value,
        );
        //dbg!(self.angular_velocity);
        //dbg!(self.signed_angular_kinetic_energy());
    }

    fn rotate(&mut self, delta_time: Time) {
        let angular_increment = self.angular_velocity * delta_time;
        self.angular_position += Angle::new::<revolution>(angular_increment.get::<ratio>());
        self.angular_position %= Angle::FULL_TURN;
    }

    pub fn update(
        &mut self,
        mechanical_acceleration_power: Power,
        electrical_deceleration_power: Power,
        delta_time: Time,
    ) {
        println!(
            "SM update: mech acc: {:.2}; elec dec: {}; dt: {}",
            mechanical_acceleration_power.into_format_args(watt, DisplayStyle::Abbreviation),
            electrical_deceleration_power.into_format_args(watt, DisplayStyle::Abbreviation),
            delta_time.into_format_args(second, DisplayStyle::Abbreviation),
        );
        self.apply_mechanical_electrical_power(
            mechanical_acceleration_power,
            electrical_deceleration_power,
            delta_time,
        );
        self.rotate(delta_time);
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

impl Display for SynchronousMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let angle = self.angular_position % Angle::FULL_TURN;
        let angle = if angle < Angle::ZERO {
            angle + Angle::FULL_TURN
        } else {
            angle
        };

        write!(
            f,
            "{} {}",
            self.angular_velocity
                .into_format_args(radian_per_second, uom::fmt::DisplayStyle::Abbreviation),
            angle.into_format_args(revolution, uom::fmt::DisplayStyle::Abbreviation)
        )
    }
}
