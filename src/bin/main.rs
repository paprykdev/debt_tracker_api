use std::str::FromStr;

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use debt_tracker_api::model::Debt;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, DateTime, Document},
    Client,
};
use serde::Serialize;

const DB_NAME: &str = "debt_tracker";
const COLLECTION_NAME: &str = "debts";

#[derive(Serialize)]
struct DebtSchema {
    name: String,
    amount: isize,
    created_at: DateTime,
    updated_at: DateTime,
}

impl From<Debt> for DebtSchema {
    fn from(debt: Debt) -> Self {
        Self {
            name: debt.name,
            amount: debt.amount,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        }
    }
}

#[get("/")]
async fn get_all_debts(client: web::Data<Client>) -> HttpResponse {
    let coll: mongodb::Collection<Debt> = client.database(DB_NAME).collection(COLLECTION_NAME);
    let mut cursor = coll.find(doc! {}).await.expect("Failed to find documents.");

    let mut debts: Vec<Document> = Vec::new();
    while let Ok(Some(debt)) = cursor.try_next().await {
        let doc = bson::to_document(&debt).expect("Failed to convert to document.");
        debts.push(doc);
    }

    let json = serde_json::to_string(&debts).expect("Failed to convert to JSON.");

    if !debts.is_empty() {
        return HttpResponse::Ok()
            .content_type("application/json")
            .body(json);
    }

    HttpResponse::Ok()
        .content_type("application/json")
        .body("[]")
}

#[get("/{id}")]
async fn get_debt_by_id(client: web::Data<Client>, id: web::Path<String>) -> HttpResponse {
    let id: ObjectId = match ObjectId::from_str(&id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID."),
    };

    let coll: mongodb::Collection<Debt> = client.database(DB_NAME).collection(COLLECTION_NAME);

    let filter = doc! {"_id": id};
    let debt = coll
        .find_one(filter)
        .await
        .expect("Failed to find document.");

    match debt {
        Some(debt) => {
            let doc = bson::to_document(&debt).expect("Failed to convert to document.");
            let json = serde_json::to_string(&doc).expect("Failed to convert to JSON.");
            HttpResponse::Ok()
                .content_type("application/json")
                .body(json)
        }
        None => HttpResponse::NotFound().body("Not found."),
    }
}

#[post("/add")]
async fn add_debt(client: web::Data<Client>, form: web::Json<Debt>) -> HttpResponse {
    let res: DebtSchema = form.into_inner().into();
    let doc = bson::to_document(&res).expect("Failed to convert to document.");
    println!("{:?}", doc);
    let collection = client.database(DB_NAME).collection(COLLECTION_NAME);
    match collection.insert_one(doc).await {
        Ok(_) => HttpResponse::Created().body("created"),
        Err(err) => HttpResponse::InternalServerError()
            .content_type("application/json")
            .body(format!("{{\"error\": \"{}\"}}", err)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb+srv://papryk:<passwd>@cluster0.yuwkm.mongodb.net/?retryWrites=true&w=majority&appName=Cluster0".to_string());

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
