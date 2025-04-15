use crate::domain::{AppError, Coordinates};
use crate::application::{GeoCodingService, WeatherService};
use serde_json::{Value, json};
use url::form_urlencoded;
use std::collections::HashMap;

pub struct GeoCodingController<S: GeoCodingService> {
    service: S,
}

impl<S: GeoCodingService> GeoCodingController<S> {
    pub fn new(service: S) -> Self {
        Self { service }
    }

    pub fn handle_request(&self, params: &HashMap<String, String>) -> Result<Value, AppError> {
        let city = params.get("city")
            .ok_or(AppError::MissingParameter("city".into()))?;

        let coords = self.service.get_coordinates(city)?;
 
        Ok(json!({
            "city": city,
            "latitude": coords.lat,
            "longitude": coords.lon
        }))
    }
}

pub struct WeatherController<S: WeatherService> {
    service: S,
}

impl<S: WeatherService> WeatherController<S> {
    pub fn new(service: S) -> Self {
        Self { service }
    }

    pub fn handle_request(&self, params: &HashMap<String, String>) -> Result<Value, AppError> {
        let lat = params.get("lat")
            .and_then(|v| v.parse::<f64>().ok())
            .ok_or(AppError::MissingParameter("lat".into()))?;

        let lon = params.get("lon")
            .and_then(|v| v.parse::<f64>().ok())
            .ok_or(AppError::MissingParameter("lon".into()))?;

        let weather = self.service.get_weather(Coordinates { lat, lon })?;
        
        Ok(json!({
            "coordinates": { "latitude": lat, "longitude": lon },
            "temperature": weather.temperature
        }))
    }
}

pub fn parse_query(query: &str) -> HashMap<String, String> {
    form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect()
}
