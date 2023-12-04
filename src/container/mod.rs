use std::fmt::Display;

use uom::fmt::DisplayStyle;
use uom::si::f64::{Area, Mass, Time, Volume};
use uom::si::mass::kilogram;
use uom::si::pressure::bar;
use uom::si::thermodynamic_temperature::degree_celsius;
use uom::si::volume::cubic_meter;
use uom::{num_traits::Zero, si::f64::Pressure};

use crate::substance::water::{self, Water};

#[derive(Debug)]
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

impl WaterContainer {
    pub fn new(volume: Volume, water: Water, steam: Water) -> Self {
        Self {
            volume,
            surface_area: Area::zero(),
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
            // First, compute the mass that should be evaporated and condensated to step towards the equillibrium.
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

            // Then, we need to take the phase change energy into account.
            // We take the energy required for evaporation out of the total body of water.
            // This may reduce the maximal evaporation mass again.

            // TODO evaporation and condensation energy

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
