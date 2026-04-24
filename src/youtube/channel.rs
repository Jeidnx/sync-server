//! Safe wrapper for YouTube's channel RSS feeds.

use chrono::{DateTime, FixedOffset};

use crate::youtube::{
    YOUTUBE_BASE_URL, YouTubeError, YouTubeResult,
    xml_helpers::{get_child_by_name, get_child_text_by_name, get_children_by_name},
};

pub struct ChannelFetcher {}

#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct ChannelRss {
    pub id: String,
    pub name: String,
    pub videos: Vec<VideoRss>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct VideoRss {
    pub id: String,
    pub title: String,
    pub description: String,
    pub published_date: DateTime<FixedOffset>,
    pub thumbnail: String,
}

impl ChannelFetcher {
    pub async fn get_channel_rss(channel_id: &str) -> YouTubeResult<ChannelRss> {
        let feed_url = format!(
            "{}/feeds/videos.xml?channel_id={}",
            YOUTUBE_BASE_URL, channel_id
        );
        let response_body = reqwest::get(feed_url)
            .await
            .map_err(|_err| YouTubeError::ConnectionError)?
            .text()
            .await
            .map_err(|_err| YouTubeError::ConnectionError)?;

        let doc = roxmltree::Document::parse(&response_body)
            .map_err(|err| YouTubeError::SyntaxError(Box::new(err)))?;

        let videos = get_children_by_name(&doc.root_element(), "entry")
            .map(|entry| -> YouTubeResult<VideoRss> {
                let media_group = get_child_by_name(&entry, "group")?;
                Ok(VideoRss {
                    id: get_child_text_by_name(&entry, "videoId")?.to_string(),
                    title: get_child_text_by_name(&entry, "title")?.to_string(),
                    published_date: get_child_text_by_name(&entry, "published").and_then(
                        |date_string| {
                            DateTime::parse_from_rfc3339(date_string).map_err(|_err| {
                                YouTubeError::ParserError("invalid date format".to_string())
                            })
                        },
                    )?,

                    description: get_child_text_by_name(&media_group, "description")?.to_string(),
                    thumbnail: get_child_by_name(&media_group, "thumbnail")?
                        .attribute("url")
                        .unwrap_or_default()
                        .to_string(),
                })
            })
            .filter_map(|v| v.ok())
            .collect::<Vec<_>>();

        Ok(ChannelRss {
            id: get_child_text_by_name(&doc.root_element(), "channelId")?.to_string(),
            name: get_child_text_by_name(&doc.root_element(), "title")?.to_string(),
            videos,
        })
    }
}
