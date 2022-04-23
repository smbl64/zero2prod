use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, connection: web::Data<PgPool>) -> HttpResponse {
    let res = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(connection.get_ref())
    .await;

    match res {
        Ok(_) => {
            log::info!("New subscription has been saved!");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            log::error!("Failed to save the record: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
