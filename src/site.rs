use std::collections::HashMap;

use crate::{api::State, feed::Feed, item::Item, tag::*};
use actix_web::{web::Query, *};
use futures::future::join_all;
use handlebars::Handlebars;
use serde::{ser::SerializeSeq, Deserialize};
use serde_json::json;
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use url::Url;

impl Feed {
    pub async fn render(&self, pool: &PgPool, hb: &Handlebars<'_>, template: &str) -> String {
        let mut items = Vec::new();
        for i in self.items(pool).await.unwrap_or_default() {
            items.push(json!({
                "id": i.id,
                "title": i.title,
            }));
        }
        let data = json!({
            "id": self.id,
            "title": self.title,
            "items": items
        });
        hb.render(template, &data).unwrap()
    }
}

impl Item {
    pub fn render(&self, hb: &Handlebars<'_>, template: &str) -> String {
        let data = json!({
            "title": self.title,
            "content": self.content
        });
        hb.render(template, &data).unwrap()
}}

#[get("/")]
pub async fn index(state: web::Data<State<'_>>) -> impl Responder {
    let hb = &state.hb;
    let pool = &state.pool;
    let feeds = Feed::feeds(pool).await.unwrap_or_default();
    let data = json!({
        "feeds": feeds.iter().map(|f| json!({"id": f.id, "title": f.title})).collect::<Vec<_>>()
    });
    let html = hb.render("html/index", &data).unwrap();
    HttpResponse::Ok().body(html)
}

#[derive(Deserialize)]
pub struct CreateFeed {
    url: String,
}

#[post("/feeds")]
pub async fn create_feed(
    form: web::Form<CreateFeed>,
    state: web::Data<State<'_>>,
) -> impl Responder {
    let pool = &state.pool;
    match Feed::add_and_update_feed(pool, &form.url).await {
        Ok(feed) => "Ok".to_string(),
        Err(e) => e.to_string(),
    }
}

#[get("/feeds/{id}")]
pub async fn get_feed(id: web::Path<i64>, state: web::Data<State<'_>>) -> impl Responder {
    let pool = &state.pool;
    let hb = &state.hb;
    let f = Feed::get_feed_by_id(pool, *id).await.unwrap();
    HttpResponse::Ok().body(f.render(pool, hb, "html/feed").await)
}

#[get("/items/{id}")]
pub async fn get_item(id: web::Path<i64>, state: web::Data<State<'_>>) -> impl Responder {
    let pool = &state.pool;
    let hb = &state.hb;
    let mut i = Item::get_item_by_id(pool, *id).await.unwrap();
    i.read = true;
    i.save(pool).await;
    HttpResponse::Ok().body(i.render(hb, "html/item"))
}

#[post("/feeds/{id}/update")]
pub async fn update_feed(id: web::Path<i64>, state: web::Data<State<'_>>) -> impl Responder {
    let pool = &state.pool;
    let hb = &state.hb;
    let mut f = Feed::get_feed_by_id(pool, *id).await.unwrap();
    let _ = f.update_feed(pool).await.unwrap();
    HttpResponse::Ok().body(f.render(pool, hb, "html/feed").await)
}

#[post("/feeds/{id}/delete")]
pub async fn delete_feed(id: web::Path<i64>, state: web::Data<State<'_>>) -> impl Responder {
    let pool = &state.pool;
    let f = Feed::get_feed_by_id(pool, *id).await.unwrap();
    f.delete(pool).await.unwrap();
    HttpResponse::Ok().body(format!("Deleted {}", f.title))
}