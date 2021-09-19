use chrono::{DateTime, NaiveDateTime};
use std::collections::HashMap;

pub struct OvercastClient(reqwest::blocking::Client);

impl OvercastClient {
    pub fn new() -> OvercastClient {
        OvercastClient(
            reqwest::blocking::Client::builder()
                .cookie_store(true)
                .build()
                .expect("reqwest client"),
        )
    }

    // Authenticates the client with Overcast. Authentication is persisted with cookies.
    pub fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = HashMap::new();
        data.insert("email", username);
        data.insert("password", password);
        let resp = self
            .0
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

    pub fn get_podcasts(&self) -> Result<Vec<Feed>, Box<dyn std::error::Error>> {
        let podcast_contents = self
            .0
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
}

#[derive(Debug)]
pub struct Feed {
    pub id: String,
    pub title: String,
    pub subscribed: bool,
    pub episodes: Vec<Episode>,
    pub feed_url: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Debug)]
pub struct Episode {
    pub id: String,
    pub title: String,
    pub played: bool,
    pub published_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub html_url: Option<String>,
    pub overcast_url: Option<String>,
    pub mp3_url: Option<String>,
    pub user_deleted: bool,
    pub progress: Option<i64>,
}
