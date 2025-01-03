use actix_web::{web, App, HttpServer};
use debt_tracker_api::routes::route::{
    add_debt, delete_debt, get_all_debts, get_debt_by_id, update_debt,
};

use dotenv::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let username = env::var("MONGO_USER").expect("MONGO_USER or MONGODB_URI must be set");
    let password = env::var("MONGO_PASSWD").expect("MONGO_PASSWD or MONGODB_URI must be set");
    let ip = env::var("IP_ADDRESS").expect("IP_ADDRESS must be set");
    let port: u16 = env::var("PORT")
        .unwrap_or("8080".into())
        .trim()
        .parse()
        .expect("PORT must be a number");

    let uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| format!("mongodb+srv://{}:{}@cluster0.yuwkm.mongodb.net/?retryWrites=true&w=majority&appName=Cluster0", username, password));

    let client = mongodb::Client::with_uri_str(&uri)
        .await
        .expect("Failed to initialize client.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(get_all_debts)
            .service(get_debt_by_id)
            .service(add_debt)
            .service(delete_debt)
            .service(update_debt)
    })
    .bind((ip, port))?
    .run()
    .await
}
