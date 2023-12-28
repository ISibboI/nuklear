use nalgebra::DMatrix;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, std::fmt::Debug)]
pub enum Error {
    #[error("the operation left some mass with a temperature below zero Kelvin")]
    NonPositiveTemperature,

    #[error("the conductance matrix used for computing the resistance network voltages is not invertible: {matrix:?}")]
    NonInvertibleConductanceMatrix { matrix: DMatrix<f64> },
}
