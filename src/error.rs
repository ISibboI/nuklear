#[derive(thiserror::Error, std::fmt::Debug)]
pub enum Error {
    #[error("the operation left some mass with a temperature below zero Kelvin")]
    NonPositiveTemperature,
}
