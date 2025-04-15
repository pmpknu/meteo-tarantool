use crate::domain::{Coordinates, WeatherInfo, AppError};

pub trait GeoCodingService: Send + Sync {
    fn get_coordinates(&self, city: &str) -> Result<Coordinates, AppError>;
}

pub trait WeatherService: Send + Sync {
    fn get_weather(&self, coordinates: Coordinates) -> Result<WeatherInfo, AppError>;
}