use actix_web_static_files::resource_dir;
use std::env;
use std::path::Path;
fn main() {
    // Embed the web server?
    if env::var("CARGO_FEATURE_EMBEDDED_WEB").is_ok() {
        let dir = Path::new("./web/dist");
        if !dir.exists() {
            panic!("The UI have not been built");
        }
        resource_dir("./web/dist").build().unwrap();
    }
}
