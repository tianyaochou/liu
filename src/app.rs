use sqlx::PgPool;
use handlebars::Handlebars;

pub struct State<'a> {
    pub pool: PgPool,
    pub hb: Handlebars<'a>,
}