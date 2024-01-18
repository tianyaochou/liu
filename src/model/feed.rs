use crate::error::Result;
use crate::model::{item::Item, tag::Tag};
use chrono::{offset, DateTime, Utc};
use md5::Digest;
use sqlx::*;

#[derive(sqlx::FromRow)]
pub struct Feed {
    pub id: i64,
    pub title: String,
    pub feed_uri: String,
    pub site_uri: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl Feed {
    pub fn new(
        id: i64,
        title: &str,
        feed_uri: &str,
        site_uri: Option<&str>,
        updated_at: DateTime<Utc>,
    ) -> Feed {
        Feed {
            id: id,
            title: title.to_string(),
            feed_uri: feed_uri.to_string(),
            site_uri: site_uri.map(|s| s.to_string()),
            updated_at: updated_at,
        }
    }

    pub async fn create(
        pool: &PgPool,
        title: &str,
        feed_uri: &str,
        site_uri: Option<&str>,
    ) -> Result<Feed> {
        let now = offset::Utc::now();
        let id = query!("insert into feeds (title, feed_uri, site_uri, updated_at) values ($1, $2, $3, $4) returning id", title, feed_uri, site_uri, now).fetch_one(pool).await?.id;
        let feed = Feed::new(id, title, feed_uri, site_uri, now);
        Ok(feed)
    }

    pub async fn create_from_feed(
        pool: &PgPool,
        uri: &str,
        feed: &feed_rs::model::Feed,
    ) -> Result<Feed> {
        let title = feed.title.clone().map(|t| t.content).unwrap_or_default();
        let site_uri = feed.links.first().map(|l| l.href.as_str());
        Self::create(pool, &title, uri, site_uri).await
    }

    pub async fn create_item_from_entry(
        &self,
        pool: &PgPool,
        entry: &feed_rs::model::Entry,
    ) -> Result<()> {
        let content = entry
            .content
            .clone()
            .map(|c| c.body.unwrap_or_default())
            .unwrap_or_default();
        let link = entry.links.first().map(|l| l.href.as_str());
        let title = entry.title.clone().map(|t| t.content).unwrap_or_default();
        let author = entry
            .authors
            .iter()
            .fold(String::new(), |acc, p| format!("{}, {}", acc, p.name));
        let created_at = entry.published.unwrap_or_default();
        let updated_at = entry.updated.unwrap_or_default();
        let _ = Item::create(
            pool, self.id, link, &title, &author, &content, created_at, updated_at,
        )
        .await;
        Ok(())
    }

    pub async fn save(&self, pool: &PgPool) -> Result<()> {
        query!(
            "update feeds set title = $1, feed_uri = $2, site_uri = $3, updated_at = $4",
            self.title,
            self.feed_uri,
            self.site_uri,
            self.updated_at
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, pool: &PgPool) -> Result<()> {
        query!("delete from feeds where id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn items(&self, pool: &PgPool) -> Result<Vec<Item>> {
        let items = query_as!(Item, "select id, feed_id, hash, link, title, author, content, created_at, updated_at, read, star from items where feed_id = $1 order by updated_at desc", self.id)
        .fetch_all(pool).await?;
        Ok(items)
    }

    pub async fn tags(&self, pool: &PgPool) -> Result<Vec<Tag>> {
        let tags = query_as!(Tag, "select tags.id, tags.name from (taggings join feeds on feeds.id = taggings.feed_id) join tags on taggings.tag_id = tags.id where feed_id = $1", self.id).fetch_all(pool).await?;
        Ok(tags)
    }

    pub async fn add_tag(&self, tag: Tag, pool: &PgPool) -> Result<()> {
        query!(
            "insert into taggings (feed_id, tag_id) values ($1, $2)",
            self.id,
            tag.id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn unread_count(&self, pool: &PgPool) -> Result<i64> {
        let count = query!(
            "select count(id) as unread_count from items where feed_id = $1 and read = false",
            self.id
        )
        .fetch_one(pool)
        .await
        .map(|row| row.unread_count.unwrap_or(0))?;
        Ok(count)
    }

    pub async fn feeds(pool: &PgPool) -> Result<Vec<Feed>> {
        let feeds = query_as!(
            Feed,
            "select id, title, feed_uri, site_uri, updated_at from feeds"
        )
        .fetch_all(pool)
        .await?;
        Ok(feeds)
    }

    pub async fn get_feed_by_id(pool: &PgPool, id: i64) -> Result<Feed> {
        let feed = query_as!(
            Feed,
            "select id, title, feed_uri, site_uri, updated_at from feeds where id = $1",
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(feed)
    }

    pub async fn get_feed_by_uri(pool: &PgPool, uri: &str) -> Result<Feed> {
        let feed = query_as!(
            Feed,
            "select id, title, feed_uri, site_uri, updated_at from feeds where feed_uri = $1",
            uri
        )
        .fetch_one(pool)
        .await?;
        Ok(feed)
    }

    pub async fn add_feed(pool: &PgPool, uri: &str, feed: &feed_rs::model::Feed) -> Result<Feed> {
        Self::create_from_feed(pool, uri, feed).await
    }

    pub async fn add_feed_from_uri(pool: &PgPool, uri: &str) -> Result<Feed> {
        let feed = get_feed(uri).await?;
        Self::add_feed(pool, uri, &feed).await
    }

    pub async fn add_and_update_feed(pool: &PgPool, uri: &str) -> Result<Feed> {
        let raw_feed = get_feed(uri).await?;
        let mut feed = Self::add_feed(pool, uri, &raw_feed).await?;
        feed.update_feed_from_feed(pool, &raw_feed).await?;
        Ok(feed)
    }

    pub async fn update_feed_from_feed(
        &mut self,
        pool: &PgPool,
        feed: &feed_rs::model::Feed,
    ) -> Result<()> {
        for entry in feed.entries.iter() {
            self.create_item_from_entry(pool, entry).await.unwrap();
        }
        Ok(())
    }

    pub async fn update_feed(&mut self, pool: &PgPool) -> Result<()> {
        let f = get_feed(&self.feed_uri).await?;
        self.update_feed_from_feed(pool, &f).await
    }
}

pub async fn get_feed(uri: &str) -> Result<feed_rs::model::Feed> {
    let src = reqwest::get(uri).await?.text().await?;
    Ok(feed_rs::parser::parse(src.as_bytes())?)
}
