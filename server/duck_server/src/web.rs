use actix_web::{get, App, HttpResponse, HttpServer, Responder};

pub async fn start() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(server_info))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

///////////////////////////////////////////////////////////
// View models

#[derive(Serialize, Clone)]
struct ServerInfoViewModel<'a> {
    pub title: &'a str,
}

///////////////////////////////////////////////////////////
// Endpoints

#[get("/api/server")]
async fn server_info() -> impl Responder {
    let view_model = ServerInfoViewModel { title: "Duck" };
    let json = serde_json::to_string(&view_model).unwrap();
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json)
}
