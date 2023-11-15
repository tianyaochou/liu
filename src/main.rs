use actix_web::*;
use liu_feed::api::*;
use liu_feed::feed;
use liu_feed::site;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPool::connect("postgres:liu-feed").await.unwrap();
    let mut hb = handlebars::Handlebars::new();
    hb.register_templates_directory(".html", "static").unwrap();
    HttpServer::new(move || {
        let data = web::Data::new(State {
            pool: pool.clone(),
            hb: hb.clone(),
        });
        let reader_api = web::scope("")
            .guard(guard::Header("content-type", "application/json"))
            .app_data(data.clone())
            .service(login)
            .service(tags);
        let site = web::scope("").app_data(data)
            .service(site::index)
            .service(site::create_feed)
            .service(site::get_feed)
            .service(site::get_item)
            .service(site::update_feed)
            .service(site::delete_feed);
        App::new()
            .wrap(middleware::Compress::default())
            .service(reader_api)
            .service(site)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
