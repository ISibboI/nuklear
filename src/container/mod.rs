use log::trace;
use std::fmt::Display;
use uom::fmt::DisplayStyle;
use uom::si::f64::{Area, Mass, Time, Volume};
use uom::si::mass::kilogram;
use uom::si::pressure::bar;
use uom::si::thermodynamic_temperature::degree_celsius;
use uom::si::volume::cubic_meter;
use uom::{num_traits::Zero, si::f64::Pressure};

use crate::error::Error;
use crate::substance::water::{self, Water};

#[derive(Debug, Clone)]
pub struct WaterContainer {
    /// The total volume of the container.
    volume: Volume,

    /// The surface area between steam and water in the container.
    /// This controls how fast the temperature between water and steam gets equalised.
    surface_area: Area,

    /// The water in the container.
    water: Water,
    /// The steam in the container.
    steam: Water,
}

pub struct PhaseEquillibrium {
    should_condensate: bool,
    should_evaporate: bool,
}

impl WaterContainer {
    pub fn new(volume: Volume, surface_area: Area, water: Water, steam: Water) -> Self {
        Self {
            volume,
            surface_area,
            water,
            steam,
        }
    }

    pub fn volume(&self) -> Volume {
        self.volume
    }

    /// The surface area between water and steam.
    /// This controls how fast temperature is convected.
    pub fn surface_area(&self) -> Area {
        self.surface_area
    }

    pub fn water_volume(&self) -> Volume {
        self.water.volume()
    }

    pub fn steam_volume(&self) -> Volume {
        self.volume - self.water_volume()
    }

    pub fn pressure(&self) -> Pressure {
        self.steam.pressure(self.steam_volume())
    }

    /// Evaporate a given mass of water into steam.
    fn evaporate(&mut self, mass: Mass) -> Result<(), Error> {
        assert!(mass >= Mass::zero() && mass <= self.water.mass());
        let additional_steam = self.water.evaporate(mass)?;
        self.steam += additional_steam;
        Ok(())
    }

    /// Condensate a given mass of steam into water.
    fn condensate(&mut self, mass: Mass) {
        assert!(mass >= Mass::zero() && mass <= self.steam.mass());
        let additional_water = self.steam.condensate(mass);
        self.water += additional_water;
    }

    pub fn phase_equillibrium(&self) -> PhaseEquillibrium {
        if self.steam.mass().is_zero() && self.water.mass().is_zero() {
            PhaseEquillibrium {
                should_condensate: false,
                should_evaporate: false,
            }
        } else {
            let steam_volume = self.steam_volume();

            if steam_volume.is_sign_negative() {
                // We assume water to be incompressible, so here steam would be at infinite pressure.
                // Hence, it should condensate all steam, and evaporate nothing.
                PhaseEquillibrium {
                    should_condensate: true,
                    should_evaporate: false,
                }
            } else {
                let pressure = self.pressure();
                let water_saturation_pressure = self.water.saturation_pressure();
                let steam_saturation_pressure = self.steam.saturation_pressure();
                PhaseEquillibrium {
                    should_condensate: pressure > steam_saturation_pressure,
                    should_evaporate: pressure < water_saturation_pressure,
                }
            }
        }
    }

    /// Evaporate and condensate water to reach the saturation pressure.
    /// Note that both processes happen simultaneously.
    /// Specifically, if the water is hot but the steam is cold, this will result in both processes happening at the same time.
    /// And, if the steam is hot but the water is cold, nothing will happen.
    ///
    /// If for some reason the volume left by the water is negative, then all steam will condensate.
    /// This is because we assume water to be incompressible.
    pub fn evaporate_condensate(&mut self) {
        let steam_volume = self.steam_volume();

        if steam_volume <= Volume::zero() {
            // We assume water to be incompressible, so here steam would be at infinite pressure.
            // Hence, we can condensate it completely.
            self.water += self.steam;
            self.steam = Water::zero();
        } else {
            // Evaporate water and condensate steam.
            // First, compute an upper bound of the mass that can be evaporated and condensated to step towards the equillibrium.
            let pressure = self.pressure();
            let water_saturation_pressure = self.water.saturation_pressure();
            let steam_saturation_pressure = self.steam.saturation_pressure();
            let water_evaporation_potential = (water_saturation_pressure - pressure) * steam_volume;
            let steam_condensation_potential =
                (pressure - steam_saturation_pressure) * steam_volume;
            let water_evaporation_mass = water_evaporation_potential
                / (water::SPECIAL_IDEAL_GAS_CONSTANT * self.water.temperature());
            let steam_condensation_mass = steam_condensation_potential
                / (water::SPECIAL_IDEAL_GAS_CONSTANT * self.steam.temperature());
            let water_evaporation_mass = water_evaporation_mass
                .max(Mass::zero())
                .min(self.water.mass());
            let steam_condensation_mass = steam_condensation_mass
                .max(Mass::zero())
                .min(self.steam.mass());

            // Then, do binary search for the actual amounts.
            let mut left = Mass::zero();
            let mut right = water_evaporation_mass;
            let mut iteration = 0;
            while (right - left) > Mass::new::<kilogram>(1e-10) {
                assert!(right > left);
                iteration += 1;

                let middle = (right + left) / 2.0;
                let mut test_container = self.clone();
                if test_container.evaporate(middle).is_err() {
                    right = middle;
                } else {
                    let phase_equillibrium = test_container.phase_equillibrium();
                    if phase_equillibrium.should_evaporate {
                        left = middle;
                    } else {
                        right = middle;
                    }
                }
            }
            let water_evaporation_mass = (right + left) / 2.0;
            trace!("Took {iteration} iterations to compute evaporation");

            let mut left = Mass::zero();
            let mut right = steam_condensation_mass;
            let mut iteration = 0;
            while (right - left) > Mass::new::<kilogram>(1e-10) {
                assert!(right > left);
                iteration += 1;

                let middle = (right + left) / 2.0;
                let mut test_container = self.clone();
                test_container.condensate(middle);
                let phase_equillibrium = test_container.phase_equillibrium();
                if phase_equillibrium.should_condensate {
                    left = middle;
                } else {
                    right = middle;
                }
            }
            let steam_condensation_mass = (right + left) / 2.0;
            trace!("Took {iteration} iterations to compute evaporation");

            self.water.simultaneous_mass_exchange(
                &mut self.steam,
                water_evaporation_mass,
                steam_condensation_mass,
            );
        }
    }

    /// Transfer heat between the steam and the water in this container.
    /// The transfer speed is dependent on the surface area parameter.
    pub fn convect(&mut self, _time: Time) {
        todo!()
    }
}

impl Display for WaterContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Volume: {:.2}; Water: {:.2} at {:.2}; Steam: {:.2} at {:.2}; Pressure: {:.4}",
            self.volume
                .into_format_args(cubic_meter, DisplayStyle::Abbreviation),
            self.water
                .mass()
                .into_format_args(kilogram, DisplayStyle::Abbreviation),
            self.water
                .temperature()
                .into_format_args(degree_celsius, DisplayStyle::Abbreviation),
            self.steam
                .mass()
                .into_format_args(kilogram, DisplayStyle::Abbreviation),
            self.steam
                .temperature()
                .into_format_args(degree_celsius, DisplayStyle::Abbreviation),
            self.pressure()
                .into_format_args(bar, DisplayStyle::Abbreviation),
        )
    }
}
