use rusqlite::{params, Connection};

use crate::overcast::Feed;

// Creates tables for podcast feeds and episodes, if they don't already exist.
pub fn create_tables(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS feeds (
            id INTEGER PRIMARY KEY,
            title TEXT,
            subscribed BOOLEAN,
            feedUrl TEXT,
            htmlUrl TEXT
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS episodes (
            id INTEGER PRIMARY KEY,
            title TEXT,
            played BOOLEAN,
            feedId INTEGER NOT NULL,
            publishedAt TEXT,
            updatedAt TEXT,
            htmlUrl TEXT,
            overcastUrl TEXT,
            mp3Url TEXT,
            progress INTEGER,
            userDeleted BOOLEAN,
            FOREIGN KEY(feedId) REFERENCES feeds(id)
        )",
        [],
    )?;
    Ok(())
}

// Upserts a list of feeds  and episodes into the database.
pub fn upsert_feeds(conn: &Connection, feeds: &[Feed]) -> Result<(), Box<dyn std::error::Error>> {
    for feed in feeds {
        conn.execute(
            "INSERT OR REPLACE INTO feeds(id, title, subscribed, feedUrl, htmlUrl)
            VALUES (?, ?, ?, ?, ?)",
            params![
                feed.id,
                feed.title,
                feed.subscribed,
                feed.feed_url,
                feed.html_url,
            ],
        )?;
        for episode in &feed.episodes {
            conn.execute(
                "INSERT OR REPLACE INTO episodes(
                    id, title, played, feedId, publishedAt, updatedAt, htmlUrl, overcastUrl, mp3Url, progress, userDeleted
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    episode.id,
                    episode.title,
                    episode.played,
                    feed.id,
                    episode.published_at,
                    episode.updated_at,
                    episode.html_url,
                    episode.overcast_url,
                    episode.mp3_url,
                    episode.progress,
                    episode.user_deleted,
                ],
            )?;
        }
    }
    Ok(())
}
