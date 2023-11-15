use crate::{
    feed::{self, Feed},
    tag::*,
};
use actix_web::{web::Query, *};
use futures::future::join_all;
use serde::{ser::SerializeSeq, Deserialize};
use serde_json::json;
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use url::Url;
use handlebars::Handlebars;

#[derive(Deserialize)]
struct LoginRequest {
    #[serde(rename = "Passwd")]
    passwd: String,
}

pub struct State<'a> {
    pub pool: PgPool,
    pub hb: Handlebars<'a>,
}

#[post("/accounts/ClientLogin")]
pub async fn login(query: web::Query<LoginRequest>) -> impl Responder {
    if true {
        // TODO: Password protection
        let session_token = "noauth"; // TODO: Authentication token
        HttpResponse::Ok().body(format!("Auth={}\n", session_token))
    } else {
        HttpResponse::Forbidden().body("Password Wrong")
    }
}

#[get("/api/0/tag/list")]
pub async fn tags(req: HttpRequest, state: web::Data<State<'_>>) -> HttpResponse {
    if !helper::check_token(&req).await {
        return HttpResponse::Forbidden().body("Authentication failed");
    }
    let stared = json!({
        "id": "user/-/state/com.google/starred"
    });
    let mut tags = vec![stared];
    Tag::tags(&state.pool)
        .await
        .unwrap_or_default()
        .iter()
        .for_each(|tag| {
            tags.push(json!({
                "id": format!("user/-/label/{}", tag.name),
                "type": "tag",
                "unread_count": 0 // TODO: unread count of tag
            }))
        });
    let response = json!({
        "tags": tags
    });
    HttpResponse::Ok().body(serde_json::to_string(&response).unwrap())
}

#[get("/api/0/subscription/list")]
pub async fn feeds(req: HttpRequest, state: web::Data<State<'_>>) -> HttpResponse {
    if !helper::check_token(&req).await {
        return HttpResponse::Forbidden().body("Authentication failed");
    }
    let feeds = Feed::feeds(&state.pool).await.unwrap_or_default();
    let feeds_json = feeds.iter().map(|f| async {
        json!({
            "id": format!("feed/{}", f.id),
            "title": f.title,
            "categories": f.tags(&state.pool).await.unwrap_or_default().iter().map(|t| json!({
                "id": format!("user/-/label/{}", t.name),
                "label": t.name
            })).collect::<Vec<JsonValue>>(),
            "url": f.feed_uri,
            "htmlUrl": f.site_uri,
            "iconUrl": null // TODO:  icon
        })
    });
    let response = json!({
        "subscriptions": join_all(feeds_json).await
    });
    HttpResponse::Ok().body(serde_json::to_string(&response).unwrap())
}

#[get("/subscriptions/export")]
pub async fn export_feeds(req: HttpRequest) -> HttpResponse {
    unimplemented!()
}

#[post("/api/0/import/opml")]
pub async fn import_feeds(req: HttpRequest) -> HttpResponse {
    unimplemented!()
}

#[derive(Deserialize)]
pub struct QuickAddQuery {
    quickadd: String,
}

#[post("/api/0/subscription/quickadd")]
pub async fn add_feed(query: Query<QuickAddQuery>, state: web::Data<State<'_>>) -> HttpResponse {
    let uri = query.quickadd.as_str();
    match Feed::add_and_update_feed(&state.pool, &uri).await {
        Ok(feed) => HttpResponse::Ok().body(json!({
            "numResults": 1,
            "query": uri,
            "streamId": format!("feed/{}", feed.id),
            "streamName": feed.title
        }).to_string()),
        Err(err) => {
            let response: JsonValue = json!({
                "numResults": 0,
                "error": "Cannot retrieve feed source"
            });
            HttpResponse::Ok().body(response.to_string())},
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum EditAction {
    Subscribe,
    Edit,
    Unsubscribe,
}

#[derive(Deserialize)]
pub struct EditFeedQuery {
    ac: EditAction,
    s: String,         // URI
    t: Option<String>, // Title
    a: Option<String>, // Add tag
    r: Option<String>, // Remove tag
}

#[post("/api/0/subscription/edit")]
pub async fn edit_feed(
    query: web::Query<EditFeedQuery>,
    req: HttpRequest,
    state: web::Data<State<'_>>,
) -> HttpResponse {
    if !helper::check_token(&req).await {
        return HttpResponse::Forbidden().body("Authentication failed");
    }
    match query.ac {
        EditAction::Subscribe => unimplemented!(),
        EditAction::Edit => unimplemented!(),
        EditAction::Unsubscribe => unimplemented!(),
    }
    unimplemented!()
}

#[get("/api/0/unread_count")]
pub async fn unread_count(req: HttpRequest, state:web::Data<State<'_>>) -> HttpResponse {
    if !helper::check_token(&req).await {
        return HttpResponse::Forbidden().body("Authentication failed");
    }
    let pool = &state.pool;
    let mut response: Vec<JsonValue> = Vec::new();
    let mut all_count = 0;
    for feed in Feed::feeds(pool).await.unwrap_or_default() {
        let count = feed.unread_count(pool).await.unwrap_or(0);
        all_count += count;
        response.push(json!({
            "id": format!("feed/{}", feed.id),
            "count": count,
            "newestItemTimestampUsec": "" // TODO: IDK
        }));
    }
    for tag in Tag::tags(pool).await.unwrap_or_default() {
        let count = tag.unread_count(pool).await.unwrap_or(0);
        response.push(json!({
            "id": format!("user/-/label/{}", tag.name),
            "count": count,
            "newestItemTimestampUsec": ""
        }))
    }
    response.push(json!({
        "id": "user/-/state/com.google/reading-list",
        "count": all_count,
        "newestItemTimestampUsec": ""
    }));
    HttpResponse::Ok().body(serde_json::to_string(&response).unwrap())
}

#[get("/api/0/stream/items/ids")]
pub async fn get_items(req: HttpRequest) -> HttpResponse {
    unimplemented!()
}

#[get("/api/0/stream/items/contents")]
pub async fn get_item_by_id(req: HttpRequest) -> HttpResponse {
    unimplemented!()
}

#[get("/api/0/stream/contents")]
pub async fn get_feed_items(req: HttpRequest) -> HttpResponse {
    unimplemented!()
}

#[post("/api/0/edit-tag")]
pub async fn editTag(req: HttpRequest) -> HttpResponse {
    unimplemented!()
}

#[get("/api/0/mark-all-as-read")]
pub async fn markRead(req: HttpRequest) -> HttpResponse {
    unimplemented!()
}

#[get("/api/0/rename-tag")]
pub async fn renameTag(req: HttpRequest) -> HttpResponse {
    unimplemented!()
}

#[get("/api/0/disable-tag")]
pub async fn remove_tag(req: HttpRequest) -> HttpResponse {
    unimplemented!()
}

mod helper {
    use actix_web::HttpRequest;

    pub async fn check_token(req: &HttpRequest) -> bool {
        let auth_token = req.headers().get("Authorization").map(|value| {
            value
                .as_bytes()
                .strip_prefix(b"GoogleLogin auth=")
                .unwrap_or_default()
        });
        match auth_token {
            Some(token) => {
                if true {
                    // TODO: authentication
                    return true;
                } else {
                    return false;
                }
            }
            None => return false,
        };
    }
}
