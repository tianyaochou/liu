use crate::error::Result;
use crate::model::feed::Feed;
use sqlx::*;

#[derive(FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

impl Tag {
    pub async fn create(pool: &PgPool, name: &str) -> Result<Tag> {
        let id = query!("insert into tags (name) values ($1) returning id", name)
            .fetch_one(pool)
            .await?
            .id;
        Ok(Tag {
            id: id,
            name: name.to_string(),
        })
    }

    pub async fn tags(pool: &PgPool) -> Result<Vec<Tag>> {
        let tags = query_as!(Tag, "select id, name from tags")
            .fetch_all(pool)
            .await?;
        Ok(tags)
    }

    pub async fn feeds(&self, pool: &PgPool) -> Result<Vec<Feed>> {
        let feeds = query_as!(Feed, "select id, title, feed_uri, site_uri, updated_at from taggings join feeds on (id = feed_id) where tag_id = $1", self.id).fetch_all(pool).await?;
        Ok(feeds)
    }

    pub async fn unread_count(&self, pool: &PgPool) -> Result<i64> {
        let feeds = self.feeds(pool).await?;
        let mut count: i64 = 0;
        for feed in feeds {
            count = count + feed.unread_count(pool).await?
        }
        Ok(count)
    }
}
