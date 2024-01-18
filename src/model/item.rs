use base64::{engine::general_purpose::STANDARD, Engine};
use feed_rs::model::Entry;
use sqlx::*;
use chrono::{DateTime, Utc};
use crate::model::feed::Feed;
use crate::error::Result;
use md5::{Md5, Digest};

#[derive(sqlx::FromRow)]
pub struct Item {
    pub id: i64,
    pub feed_id: i64,
    pub hash: String,
    pub link: Option<String>,
    pub title: String,
    pub author: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub read: bool,
    pub star: bool
}

impl Item {
    pub async fn feed(&self, pool: &PgPool) -> Result<Feed> {
        Feed::get_feed_by_id(pool, self.feed_id).await
    }

    pub async fn create(pool: &PgPool, feed_id: i64, link: Option<&str>, title: &str, author: &str, content: &str, created_at: DateTime<Utc>, updated_at: DateTime<Utc>) -> Result<Item> {
        let mut hasher = Md5::new();
        hasher.update(title.as_bytes());
        hasher.update(content.as_bytes());
        let hash = hasher.finalize();
        let hash_text = STANDARD.encode(hash);
        let id = query!("insert into items (feed_id, hash, title, author, content, created_at, updated_at) values ($1, $2, $3, $4, $5, $6, $7) returning id", feed_id, hash_text.as_str(), title, author, content, created_at, updated_at).fetch_one(pool).await?.id;
        Ok(Item {id, feed_id, hash: hash_text, link: link.map(|x| x.to_string()), title: title.to_string(), author: author.to_string(), content: content.to_string(), created_at, updated_at, star: false, read: false })
    }

    pub async fn save(&self, pool: &PgPool) -> Result<()> {
        query!("update items set read = $1, star = $2 where id = $3", self.read, self.star, self.id).execute(pool).await?;
        Ok(())
    }

    pub async fn unread_count(pool: &PgPool) -> Result<i64> {
        let count = query!("select count(id) as unread_count from items where read = false").fetch_one(pool).await.map(|row| row.unread_count.unwrap_or(0))?;
        Ok(count)
    }

    pub async fn get_item_by_id(pool: &PgPool, id: i64) -> Result<Item> {
        let item = query_as!(Item, "select id, feed_id, hash, link, title, author, content, created_at, updated_at, read, star from items where id = $1", id).fetch_one(pool).await?;
        Ok(item)
    }
}