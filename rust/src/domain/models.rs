use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Serialize)]
pub struct WeatherInfo {
    pub temperature: String,
}