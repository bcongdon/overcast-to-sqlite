use chrono::{DateTime, NaiveDateTime};
use clap::{AppSettings, Clap};
use reqwest::blocking::Client;
use rusqlite::{params, Connection};
use std::collections::HashMap;

#[derive(Clap)]
#[clap(version = "1.0", author = "Ben Congdon <ben@congdon.dev>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Overcast username.
    #[clap(short, long)]
    username: String,
    /// Overcast password.
    #[clap(short, long)]
    password: String,
    /// The sqlite database path to store to.
    db_path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();
    let client = Client::builder().cookie_store(true).build().unwrap();

    eprintln!("[1/3] Authenticating with Overcast...");
    authenticate(&client, &opts.username, &opts.password)?;
    eprintln!("[2/3] Fetching podcasts...");
    let podcasts = get_podcasts(&client)?;
    eprintln!(
        "Fetched {} feeds with a total of {} episodes.",
        podcasts.len(),
        podcasts.iter().map(|p| p.episodes.len()).sum::<usize>()
    );

    eprintln!("[3/3] Writing podcasts to sqlite db...");
    let conn = Connection::open(&opts.db_path)?;
    create_tables(&conn)?;
    upsert_feeds(&conn, &podcasts)?;
    Ok(())
}

fn create_tables(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
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

fn upsert_feeds(conn: &Connection, feeds: &Vec<Feed>) -> Result<(), Box<dyn std::error::Error>> {
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

fn authenticate(
    client: &Client,
    username: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut data = HashMap::new();
    data.insert("email", username);
    data.insert("password", password);
    let resp = client
        .post("https://overcast.fm/login")
        .form(&data)
        .send()?;
    if resp
        .text()?
        .contains(&"Sorry, there was a problem looking up your Overcast account".to_string())
    {
        return Err("unable to authenticate with Overcast")?;
    }
    Ok(())
}

#[derive(Debug)]
struct Feed {
    id: String,
    title: String,
    subscribed: bool,
    episodes: Vec<Episode>,
    feed_url: Option<String>,
    html_url: Option<String>,
}

#[derive(Debug)]
struct Episode {
    id: String,
    title: String,
    played: bool,
    published_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    html_url: Option<String>,
    overcast_url: Option<String>,
    mp3_url: Option<String>,
    user_deleted: bool,
    progress: Option<i64>,
}

fn get_podcasts(client: &Client) -> Result<Vec<Feed>, Box<dyn std::error::Error>> {
    let podcast_contents = client
        .get("https://overcast.fm/account/export_opml/extended")
        .send()?
        .text()?;
    let tree = roxmltree::Document::parse(&podcast_contents)?;
    let feeds = tree
        .descendants()
        .find(|n| n.tag_name().name() == "outline" && n.attribute("text") == Some("feeds"))
        .unwrap();

    let mut out = Vec::new();
    for feed in feeds.children() {
        let title = feed.attribute("title");
        let id = feed.attribute("overcastId");
        if title.is_none() || id.is_none() {
            continue;
        }

        let mut episodes = Vec::new();
        for episode in feed.children() {
            if let [Some(title), Some(id)] =
                [episode.attribute("title"), episode.attribute("overcastId")]
            {
                episodes.push(Episode {
                    id: id.to_string(),
                    played: episode.attribute("played") == Some("1"),
                    title: title.to_string(),
                    updated_at: episode.attribute("userUpdatedDate").and_then(|u| {
                        DateTime::parse_from_rfc3339(u)
                            .map(|d| d.naive_local())
                            .ok()
                    }),
                    published_at: episode.attribute("pubDate").and_then(|u| {
                        DateTime::parse_from_rfc3339(u)
                            .map(|d| d.naive_local())
                            .ok()
                    }),
                    mp3_url: episode.attribute("enclosureUrl").map(|s| s.to_string()),
                    overcast_url: episode.attribute("overcastUrl").map(|s| s.to_string()),
                    html_url: episode.attribute("url").map(|s| s.to_string()),
                    progress: episode
                        .attribute("progress")
                        .and_then(|p| p.parse::<i64>().ok()),
                    user_deleted: episode.attribute("userDeleted") == Some("1"),
                });
            }
        }
        out.push(Feed {
            id: id.unwrap().to_string(),
            title: title.unwrap().to_string(),
            subscribed: feed.attribute("subscribed") == Some("1"),
            episodes: episodes,
            feed_url: feed.attribute("xmlUrl").map(|s| s.to_string()),
            html_url: feed.attribute("htmlUrl").map(|s| s.to_string()),
        });
    }
    Ok(out)
}