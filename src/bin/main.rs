use actix_web::{web, App, HttpServer};
use debt_tracker_api::routes::route::{add_debt, get_all_debts, get_debt_by_id};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| format!("mongodb+srv://papryk:<passwd>@cluster0.yuwkm.mongodb.net/?retryWrites=true&w=majority&appName=Cluster0"));

    let client = mongodb::Client::with_uri_str(&uri)
        .await
        .expect("Failed to initialize client.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(get_all_debts)
            .service(get_debt_by_id)
            .service(add_debt)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
