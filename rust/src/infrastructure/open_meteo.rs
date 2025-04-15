use super::super::application::{GeoCodingService, WeatherService};
use super::super::domain::{Coordinates, WeatherInfo, AppError};
use serde_json::Value;
use fibreq;

pub struct OpenMeteoGeoCodingService;

impl GeoCodingService for OpenMeteoGeoCodingService {
    fn get_coordinates(&self, city: &str) -> Result<Coordinates, AppError> {
        let url = format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
            city
        );

        let client = fibreq::ClientBuilder::new().build();
        let mut response = client.get(&url)
            .map_err(|e| AppError::ServiceError(format!("Request failed: {}", e)))?
            .send()
            .map_err(|e| AppError::ServiceError(format!("Request failed: {}", e)))?;

        let text = response.text()
            .map_err(|e| AppError::ServiceError(format!("Response error: {}", e)))?;

        let json: Value = serde_json::from_str(&text)?;

        json.get("results")
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .and_then(|result| {
                let lat = result.get("latitude")?.as_f64()?;
                let lon = result.get("longitude")?.as_f64()?;
                Some(Coordinates { lat, lon })
            })
            .ok_or_else(|| AppError::ServiceError("No coordinates found".into()))
    }
}

pub struct OpenMeteoWeatherService;

impl WeatherService for OpenMeteoWeatherService {
    fn get_weather(&self, coordinates: Coordinates) -> Result<WeatherInfo, AppError> {
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m&forecast_days=1",
            coordinates.lat, coordinates.lon
        );

        let client = fibreq::ClientBuilder::new().build();
        let mut response = client.get(&url)
            .map_err(|e| AppError::ServiceError(format!("Request failed: {}", e)))?
            .send()
            .map_err(|e| AppError::ServiceError(format!("Request failed: {}", e)))?;

        let text = response.text()
            .map_err(|e| AppError::ServiceError(format!("Response error: {}", e)))?;

        let json: Value = serde_json::from_str(&text)?;

        json.get("hourly")
            .and_then(|h| h.get("temperature_2m"))
            .and_then(|t| t.as_array())
            .and_then(|temps| temps.first())
            .and_then(|temp| temp.as_f64())
            .map(|t| WeatherInfo { temperature: format!("{} °C", t) })
            .ok_or_else(|| AppError::ServiceError("Temperature data not found".into()))
    }
}
