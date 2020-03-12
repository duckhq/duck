use std::env;
use std::path::Path;
use actix_web_static_files::resource_dir;
fn main() {
    // Embed the web server?
    if let Ok(_) = env::var("CARGO_FEATURE_EMBEDDED_WEB") {
        let dir = Path::new("./web/dist");
        if !dir.exists() {
            panic!("The UI have not been built.");
        }
        resource_dir("./web/dist").build().unwrap();
    }
}