pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;

#[tarantool::proc]
pub fn make_http_endpoints() {
    presentation::routes::make_http_endpoints();
}