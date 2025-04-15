use shors::transport::http::route::Builder;
use shors::transport::http::{server, Request};
use shors::transport::Context;
use crate::infrastructure::open_meteo::{OpenMeteoGeoCodingService, OpenMeteoWeatherService};
use crate::presentation::controllers::{GeoCodingController, WeatherController, parse_query};
use std::error::Error;
use serde_json::Value;

pub fn make_http_endpoints() {
    let geocoding_service = OpenMeteoGeoCodingService;
    let weather_service = OpenMeteoWeatherService;

    let geocoding_controller = GeoCodingController::new(geocoding_service);
    let weather_controller = WeatherController::new(weather_service);

    let route_group = Builder::new()
        .with_path("/api")
        .with_middleware(|route| {
            println!("got new http request to /api endpoint!");
            route
        })
        .group();

    // Geocoding endpoint
    let geocoding = route_group
        .builder()
        .with_method("GET")
        .with_path("/geocoding")
        .build(move |_ctx: &mut Context, request: Request| -> Result<Value, Box<dyn Error>> {
            let params = parse_query(&request.query);
            geocoding_controller.handle_request(&params)
                .map_err(|e| Box::new(e) as Box<dyn Error>)
        });

    // Weather endpoint
    let weather = route_group
        .builder()
        .with_method("GET")
        .with_path("/weather")
        .build(move |_ctx: &mut Context, request: Request| -> Result<Value, Box<dyn Error>> {
            let params = parse_query(&request.query);
            weather_controller.handle_request(&params)
                .map_err(|e| Box::new(e) as Box<dyn Error>)
        });

    // Ping endpoint
    let ping = route_group
        .builder()
        .with_method("GET")
        .with_path("/ping")
        .build(|_ctx: &mut Context, _request: Request| -> Result<_, Box<dyn Error>> { 
            Ok("pong") 
        });

    let s = server::Server::new();
    s.register(Box::new(geocoding));
    s.register(Box::new(weather));
    s.register(Box::new(ping));
}

