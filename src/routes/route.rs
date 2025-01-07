use std::str::FromStr;

use actix_web::{delete, get, post, put, web, HttpResponse};
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, DateTime},
    Client,
};
use serde::Serialize;

use crate::models::debt::Debt;

const DB_NAME: &str = "debt_tracker";
const COLLECTION_NAME: &str = "debts";

#[derive(Serialize)]
#[allow(non_snake_case)]
struct DebtReturnType {
    _id: String,
    name: String,
    amount: isize,
    createdAt: String,
    updatedAt: String,
}

impl From<Debt> for DebtReturnType {
    fn from(debt: Debt) -> Self {
        Self {
            _id: debt.id.unwrap().to_string(),
            name: debt.name,
            amount: debt.amount,
            createdAt: debt.created_at.into_valid_datetime(),
            updatedAt: debt.updated_at.into_valid_datetime(),
        }
    }
}

trait IntoValidDatetime {
    fn into_valid_datetime(&self) -> String;
}

impl IntoValidDatetime for Option<DateTime> {
    fn into_valid_datetime(&self) -> String {
        self.unwrap_or(DateTime::now())
            .to_string()
            .replacen(" ", "T", 1)
            .replacen(" ", "", 1)
    }
}

#[get("/")]
async fn get_all_debts(client: web::Data<Client>) -> HttpResponse {
    let coll: mongodb::Collection<Debt> = client.database(DB_NAME).collection(COLLECTION_NAME);
    let mut cursor = coll.find(doc! {}).await.expect("Failed to find documents.");

    let mut debts: Vec<DebtReturnType> = Vec::new();
    while let Ok(Some(debt)) = cursor.try_next().await {
        let debt: DebtReturnType = debt.into();
        debts.push(debt);
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
    let oid: ObjectId = match ObjectId::from_str(&id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID."),
    };

    println!("{:?}", oid);
    let coll: mongodb::Collection<Debt> = client.database(DB_NAME).collection(COLLECTION_NAME);

    let filter = doc! {"_id": oid};
    let debt = coll
        .find_one(filter)
        .await
        .expect("Failed to find document.");

    match debt {
        Some(debt) => {
            let debt: DebtReturnType = debt.into();
            let debt = serde_json::to_string(&debt).expect("Failed to convert to JSON.");
            HttpResponse::Ok()
                .content_type("application/json")
                .body(debt)
        }
        None => HttpResponse::NotFound().body("Not found."),
    }
}

#[post("/add")]
async fn add_debt(client: web::Data<Client>, form: web::Json<Debt>) -> HttpResponse {
    let mut res = form.into_inner();
    res.created_at = Some(DateTime::now());
    res.updated_at = Some(DateTime::now());
    let collection = client.database(DB_NAME).collection(COLLECTION_NAME);
    match collection.insert_one(res.clone()).await {
        Ok(debt) => {
            res.id = Some(debt.inserted_id.as_object_id().unwrap());
            let debt: DebtReturnType = res.into();
            let debt = serde_json::to_string(&debt).expect("Failed to convert to JSON.");
            HttpResponse::Created()
                .content_type("application/json")
                .body(debt)
        }
        Err(err) => HttpResponse::InternalServerError()
            .content_type("application/json")
            .body(format!("{{\"error\": \"{}\"}}", err)),
    }
}

#[delete("/{id}")]
async fn delete_debt(client: web::Data<Client>, id: web::Path<String>) -> HttpResponse {
    let oid: ObjectId = match ObjectId::from_str(&id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID."),
    };

    let coll: mongodb::Collection<Debt> = client.database(DB_NAME).collection(COLLECTION_NAME);

    let filter = doc! {"_id": oid};
    let res = coll.delete_one(filter).await;

    match res {
        Ok(debt) => {
            let debt = serde_json::to_string(&debt).expect("Failed to convert to JSON.");
            HttpResponse::Ok()
                .content_type("application/json")
                .body(debt)
        }
        Err(err) => HttpResponse::InternalServerError()
            .content_type("application/json")
            .body(format!("{{\"error\": \"{}\"}}", err)),
    }
}

#[put("/{id}")]
async fn update_debt(
    client: web::Data<Client>,
    id: web::Path<String>,
    form: web::Json<Debt>,
) -> HttpResponse {
    let mut req = form.into_inner();
    let oid: ObjectId = match ObjectId::from_str(&id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID."),
    };
    req.updated_at = Some(DateTime::now());

    let coll: mongodb::Collection<Debt> = client.database(DB_NAME).collection(COLLECTION_NAME);

    let filter = doc! {"_id": oid};
    let update = doc! {"$set": bson::to_document(&req).expect("naah")};

    match coll.update_one(filter, update).await {
        Ok(_) => HttpResponse::Ok().body("updated"),
        Err(err) => HttpResponse::InternalServerError()
            .content_type("application/json")
            .body(format!("{{\"error\": \"{}\"}}", err)),
    }
}
