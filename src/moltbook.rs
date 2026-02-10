use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};

const MOLTBOOK_API_BASE: &str = "https://www.moltbook.com/api/v1";

/// Moltbook post structure
#[derive(Debug, Deserialize, Serialize)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub submolt: Option<String>,
    #[serde(default)]
    pub upvotes: i32,
    #[serde(default)]
    pub downvotes: i32,
    #[serde(default)]
    pub comment_count: i32,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Author info from API
#[derive(Debug, Deserialize)]
struct AuthorInfo {
    name: String,
}

/// Submolt info from API
#[derive(Debug, Deserialize)]
struct SubmoltInfo {
    name: String,
}

/// Raw post from API
#[derive(Debug, Deserialize)]
struct RawPost {
    id: String,
    title: String,
    #[serde(default)]
    content: String,
    author: Option<AuthorInfo>,
    submolt: Option<SubmoltInfo>,
    #[serde(default)]
    upvotes: i32,
    #[serde(default)]
    downvotes: i32,
    #[serde(default)]
    comment_count: i32,
    created_at: Option<String>,
}

impl From<RawPost> for Post {
    fn from(raw: RawPost) -> Self {
        Post {
            id: raw.id,
            title: raw.title,
            content: raw.content,
            author: raw.author.map(|a| a.name),
            submolt: raw.submolt.map(|s| s.name),
            upvotes: raw.upvotes,
            downvotes: raw.downvotes,
            comment_count: raw.comment_count,
            created_at: raw.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
struct FeedResponse {
    success: bool,
    posts: Option<Vec<RawPost>>,
    error: Option<String>,
}

/// Client for interacting with Moltbook API
pub struct MoltbookClient {
    client: reqwest::Client,
    api_key: String,
}

impl MoltbookClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, api_key }
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let auth_value = format!("Bearer {}", self.api_key);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).expect("Invalid API key format"),
        );
        headers
    }

    /// Fetch the feed with specified sort and limit
    pub async fn get_feed(&self, sort: &str, limit: u32) -> Result<Vec<Post>, String> {
        let url = format!("{}/posts?sort={}&limit={}", MOLTBOOK_API_BASE, sort, limit);

        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()));
        }

        let feed: FeedResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if !feed.success {
            return Err(feed.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        Ok(feed.posts
            .unwrap_or_default()
            .into_iter()
            .map(Post::from)
            .collect())
    }

    /// Fetch a specific post by ID
    pub async fn get_post(&self, post_id: &str) -> Result<Post, String> {
        let url = format!("{}/posts/{}", MOLTBOOK_API_BASE, post_id);

        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct PostResponse {
            success: bool,
            post: Option<RawPost>,
            error: Option<String>,
        }

        let resp: PostResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if !resp.success {
            return Err(resp.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        resp.post
            .map(Post::from)
            .ok_or_else(|| "Post not found".to_string())
    }

    /// Get personalized feed (from subscriptions + following)
    pub async fn get_personalized_feed(&self, sort: &str, limit: u32) -> Result<Vec<Post>, String> {
        let url = format!("{}/feed?sort={}&limit={}", MOLTBOOK_API_BASE, sort, limit);

        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()));
        }

        let feed: FeedResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if !feed.success {
            return Err(feed.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        Ok(feed.posts
            .unwrap_or_default()
            .into_iter()
            .map(Post::from)
            .collect())
    }
}
