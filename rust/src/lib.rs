use shors::transport::http::route;
use shors::transport::http::{server, Request};
use shors::transport::Context;
use shors::transport::rpc::client;
use tarantool::index::{IndexOptions, IndexType};
use tarantool::space::{Space, SpaceCreateOptions, SpaceEngineType};
use std::error::Error;
use tarantool::proc;
use serde_json::{Value, json};
use url::form_urlencoded;
use std::collections::HashMap;

// @param city
// @return latitude, longitude
fn api_get_geocoding(city: &str) -> Result<(f64, f64), String> {
    let api_string = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        city
    );

    let client = fibreq::ClientBuilder::new().build();
    let mut response = match client.get(&api_string).unwrap().send() {
        Ok(resp) => resp,
        Err(e) => return Err(format!("Failed to send request: {}", e))
    };

    println!("Status: {}", response.status());
    if let Ok(text) = response.text() {
        println!("Body: {}", text);
        
        let json: Value = match serde_json::from_str(&text) {
            Ok(j) => j,
            Err(e) => return Err(format!("Failed to parse JSON: {}", e))
        };

        if let Some(results) = json.get("results").and_then(|r| r.as_array()) {
            if !results.is_empty() {
                if let (Some(lat), Some(lon)) = (
                    results[0].get("latitude").and_then(|l| l.as_f64()),
                    results[0].get("longitude").and_then(|l| l.as_f64())
                ) {
                    return Ok((lat, lon));
                }
            }
        }
        return Err("No coordinates found in response".to_string());
    }
    return Err("Failed to get response text".to_string());
}

fn create_geocoding_space() -> Result<(), String> {
    let space = Space::create(
        "geocoding_cache",
        &SpaceCreateOptions {
            engine: SpaceEngineType::Memtx,
            if_not_exists: true,
            format: Some(vec![
                tarantool::space::Field::string("city"),
                tarantool::space::Field::array("coordinates"),
            ]),
            ..SpaceCreateOptions::default()
        },
    ).map_err(|e| format!("Failed to create geocoding_cache space: {}", e))?;

    if let Err(e) = space.create_index(
        "primary",
        &IndexOptions {
            r#type: Some(IndexType::Hash),
            unique: Some(true),
            parts: Some(vec![tarantool::index::Part::field(1)]),
            if_not_exists: Some(true),
            ..IndexOptions::default()
        }
    ) {
        return Err(format!("Failed to create index: {}", e));
    }

    Ok(())
}

// @param city
// @return latitude, longitude
fn get_geocoding(city: &str) -> Result<(f64, f64), String> {
    if let Some(space) = Space::find("geocoding_cache") {
        if let Ok(Some(geocoding)) = space.get(&(city,)) {
            if let Ok(coords) = geocoding.decode::<Vec<f64>>() {
                if coords.len() >= 2 {
                    println!("CACHED Coordinates for {}: lat: {}, lon: {}", city, coords[0], coords[1]);
                    return Ok((coords[0], coords[1]));
                }
            }
        }

        let (lat, lon) = api_get_geocoding(city)?;
        println!("API Coordinates for {}: lat: {}, lon: {}", city, lat, lon);

        if let Err(e) = space.put(&(city, vec![lat, lon])) {
            println!("Failed to cache coordinates for {}: {}", city, e);
        }
        Ok((lat, lon))
    } else {
        println!("Failed to find geocoding_cache space");
        if let Err(e) = create_geocoding_space() {
            println!("Failed to create geocoding_cache space: {}", e);
            return Err(e);
        }
        let (lat, lon) = api_get_geocoding(city)?;
        println!("API Coordinates for {}: lat: {}, lon: {}", city, lat, lon);
        
        if let Some(space) = Space::find("geocoding_cache") {
            if let Err(e) = space.put(&(city, vec![lat, lon])) {
                println!("Failed to cache coordinates for {}: {}", city, e);
            }
        } else {
            println!("Space not found after creation");
        }
        Ok((lat, lon))
    }
}
pub fn upsert_geocoding(city: &str) -> Result<serde_json::Value, String> {
  let bucket_id =  1;// по широте+долгота;
  let lua = tarantool::lua_state();
  
  let mut ctx = Context::new(); // Create a new Context instance
  let resp = client::Builder::new(&lua)
                  .shard_endpoint("get_geocoding")
                  .call(&mut ctx, bucket_id, city);

  match resp {
      Ok(tuple) => {
          let city: String = tuple.get(0).unwrap_or_default();
          let latitude: f64 = tuple.get(1).unwrap_or_default();
          let longitude: f64 = tuple.get(2).unwrap_or_default();

          Ok(json!({
              "city": city,
              "latitude": latitude,
              "longitude": longitude
          }))
      },
      Err(e) => Err(format!("Failed to call remote procedure: {}", e))
  }
}

// @param latitude, longitudepace::find("geocoding_cache").pk.
// @return temperature
fn get_weather(latitude: f64, longitude: f64) -> Result<String, String> {
    let api_string = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m&forecast_days=1",
        latitude, longitude
    );

    let client = fibreq::ClientBuilder::new().build();
    let mut response = match client.get(&api_string).unwrap().send() {
        Ok(resp) => resp,
        Err(e) => return Err(format!("Failed to send request: {}", e))
    };

    println!("Status: {}", response.status());
    if let Ok(text) = response.text() {
        println!("Body: {}", text);
        
        let json: Value = match serde_json::from_str(&text) {
            Ok(j) => j,
            Err(e) => return Err(format!("Failed to parse JSON: {}", e))
        };

        if let Some(temps) = json.get("hourly").and_then(|h| h.get("temperature_2m")).and_then(|t| t.as_array()) {
            if !temps.is_empty() {
                if let Some(temp) = temps[0].as_f64() {
                    return Ok(format!("{} °C", temp));
                }
            }
        }
        return Err("Temperature data not found in response".to_string());
    }
    return Err("Failed to get response text".to_string());
}

#[proc]
pub fn make_http_endpoints() {
    let route_group = route::Builder::new()
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
    .build(
        |_ctx: &mut Context, request: Request| -> Result<serde_json::Value, Box<dyn Error>> {
            println!("RUST geocoding Request: {:?}", request.query);

            let query_string = &request.query;

            let parsed: HashMap<String, String> = 
                form_urlencoded::parse(query_string.as_bytes())
                .into_owned()
                .collect();

            if let Some(city) = parsed.get("city") {
                println!("Город: {}", city);

                match get_geocoding(city) {
                    Ok((lat, lon)) => {
                        let response = json!({
                            "city": city,
                            "latitude": lat,
                            "longitude": lon
                        });
                        Ok(response)
                    },
                    Err(e) => {
                        Err(format!("Geocoding error: {}", e).into())
                    }
                }
            } else {
                println!("Параметр 'city' не найден");
                Err("Missing city parameter".into())
            }
        },
    );

    // Weather endpoint
    let weather = route_group
        .builder()
        .with_method("GET")
        .with_path("/weather")
        .build(
            |_ctx: &mut Context, request: Request| -> Result<serde_json::Value, Box<dyn Error>> {
                println!("RUST weather Request: {request:?}");

                let query_string = &request.query;
                
                let parsed: HashMap<String, String> = 
                    form_urlencoded::parse(query_string.as_bytes())
                    .into_owned()
                    .collect();

            if let Some(lat) = parsed.get("lat").and_then(|l| l.parse::<f64>().ok()) {
                if let Some(lon) = parsed.get("lon").and_then(|l| l.parse::<f64>().ok()) {
                    println!("Широта: {}, Долгота: {}", lat, lon);

                    match get_weather(lat, lon) {
                        Ok(temp) => {
                            let response = json!({
                                "coordinates": {
                                    "latitude": lat,
                                    "longitude": lon
                                },
                                "temperature": temp
                            });
                            Ok(response)
                        },
                        Err(e) => Err(format!("Weather error: {}", e).into())
                    }
                } else {
                    Err("Missing 'lon' parameter".into())
                }
            } else {
                Err("Missing 'lat' parameter".into())
            }
            },
        );

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