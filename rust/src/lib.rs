use tarantool::proc;

#[proc]
fn get_api_string() -> String {
    let api_string = "https://api.open-meteo.com/v1/forecast?latitude=55.7558&longitude=37.6176&current=temperature_2m";
    
    let client = fibreq::ClientBuilder::new().build();
    let mut response = client.get(api_string).unwrap().send().unwrap();

    println!("Status: {}", response.status());
    if response.status().is_success() {
        response.text().unwrap()
    } else {
        "bad request".to_string()
    }
}
