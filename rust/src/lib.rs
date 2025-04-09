use tarantool::proc;

// @param city
// @return latitude, longitude
#[proc]
fn get_geocoding(city: &str) -> Result<(f64, f64), String> {
    let api_string = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        city
    );

    let client = fibreq::ClientBuilder::new().build();
    let mut response = client.get(&api_string).unwrap().send().unwrap();

    println!("Status: {}", response.status());
    if let Ok(text) = response.text() {
        println!("Body: {}", text);
        return Ok((30.0, 30.0));
    }
    return Err("bad :(".to_string());
}

// @param latitude, longitude
// @return temperature
#[proc]
fn get_weather(latitude: f64, longitude: f64) -> Result<String, String> {
    let api_string = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m&forecast_days=1",
        latitude, longitude
    );

    let client = fibreq::ClientBuilder::new().build();
    let mut response = client.get(&api_string).unwrap().send().unwrap();

    println!("Status: {}", response.status());
    if let Ok(text) = response.text() {
        println!("Body: {}", text);
        return Ok(text);
    }
    return Err("bad :(".to_string());
}