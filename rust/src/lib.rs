use tarantool::proc;

#[proc]
fn get_api_string() -> String {
    let api_string = "http://api.openweathermap.org/data/2.5/weather?q=London&appid=your_api_key";
    api_string.to_string()
}
