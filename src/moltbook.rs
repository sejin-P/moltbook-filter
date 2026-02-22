use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

const MOLTBOOK_API_BASE: &str = "https://www.moltbook.com/api/v1";

/// User profile structure
#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub karma: i32,
    #[serde(default)]
    pub followers: i32,
    #[serde(default)]
    pub following: i32,
    #[serde(default)]
    pub post_count: i32,
    #[serde(default)]
    pub comment_count: i32,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Comment structure
#[derive(Debug, Deserialize, Serialize)]
pub struct Comment {
    pub id: String,
    pub content: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub upvotes: i32,
    #[serde(default)]
    pub created_at: Option<String>,
}

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

    /// Create a new post
    pub async fn create_post(&self, title: &str, content: &str, submolt: Option<&str>) -> Result<Post, String> {
        let url = format!("{}/posts", MOLTBOOK_API_BASE);

        #[derive(Serialize)]
        struct CreatePostRequest<'a> {
            title: &'a str,
            content: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            submolt_name: Option<&'a str>,
        }

        let body = CreatePostRequest { title, content, submolt_name: submolt };

        let mut headers = self.auth_headers();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API returned status {}: {}", status, body));
        }

        #[derive(Deserialize)]
        struct CreatePostResponse {
            success: bool,
            post: Option<RawPost>,
            error: Option<String>,
        }

        let resp: CreatePostResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if !resp.success {
            return Err(resp.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        resp.post
            .map(Post::from)
            .ok_or_else(|| "No post in response".to_string())
    }

    /// Upvote a post
    pub async fn upvote(&self, post_id: &str) -> Result<(), String> {
        self.vote(post_id, "upvote").await
    }

    /// Downvote a post
    pub async fn downvote(&self, post_id: &str) -> Result<(), String> {
        self.vote(post_id, "downvote").await
    }

    /// Remove vote from a post
    pub async fn unvote(&self, post_id: &str) -> Result<(), String> {
        self.vote(post_id, "unvote").await
    }

    async fn vote(&self, post_id: &str, action: &str) -> Result<(), String> {
        let url = format!("{}/posts/{}/{}", MOLTBOOK_API_BASE, post_id, action);

        let response = self
            .client
            .post(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API returned status {}: {}", status, body));
        }

        #[derive(Deserialize)]
        struct VoteResponse {
            success: bool,
            error: Option<String>,
        }

        let resp: VoteResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if !resp.success {
            return Err(resp.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        Ok(())
    }

    /// Add a comment to a post
    pub async fn comment(&self, post_id: &str, content: &str) -> Result<Comment, String> {
        let url = format!("{}/posts/{}/comments", MOLTBOOK_API_BASE, post_id);

        #[derive(Serialize)]
        struct CommentRequest<'a> {
            content: &'a str,
        }

        let body = CommentRequest { content };

        let mut headers = self.auth_headers();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API returned status {}: {}", status, body));
        }

        #[derive(Deserialize)]
        struct RawComment {
            id: String,
            content: String,
            author: Option<AuthorInfo>,
            #[serde(default)]
            upvotes: i32,
            created_at: Option<String>,
        }

        #[derive(Deserialize)]
        struct CommentResponse {
            success: bool,
            comment: Option<RawComment>,
            error: Option<String>,
        }

        let resp: CommentResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if !resp.success {
            return Err(resp.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        resp.comment
            .map(|c| Comment {
                id: c.id,
                content: c.content,
                author: c.author.map(|a| a.name),
                upvotes: c.upvotes,
                created_at: c.created_at,
            })
            .ok_or_else(|| "No comment in response".to_string())
    }

    /// Get comments on a post
    pub async fn get_comments(&self, post_id: &str) -> Result<Vec<Comment>, String> {
        let url = format!("{}/posts/{}/comments", MOLTBOOK_API_BASE, post_id);

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
        struct RawComment {
            id: String,
            content: String,
            author: Option<AuthorInfo>,
            #[serde(default)]
            upvotes: i32,
            created_at: Option<String>,
        }

        #[derive(Deserialize)]
        struct CommentsResponse {
            success: bool,
            comments: Option<Vec<RawComment>>,
            error: Option<String>,
        }

        let resp: CommentsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if !resp.success {
            return Err(resp.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        Ok(resp.comments
            .unwrap_or_default()
            .into_iter()
            .map(|c| Comment {
                id: c.id,
                content: c.content,
                author: c.author.map(|a| a.name),
                upvotes: c.upvotes,
                created_at: c.created_at,
            })
            .collect())
    }

    /// Get the authenticated user's profile
    pub async fn get_my_profile(&self) -> Result<Profile, String> {
        let url = format!("{}/users/me", MOLTBOOK_API_BASE);

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
        struct ProfileResponse {
            success: bool,
            user: Option<Profile>,
            error: Option<String>,
        }

        let resp: ProfileResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if !resp.success {
            return Err(resp.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        resp.user
            .ok_or_else(|| "No user in response".to_string())
    }

    /// Get a user's profile by name
    pub async fn get_profile(&self, username: &str) -> Result<Profile, String> {
        let url = format!("{}/users/{}", MOLTBOOK_API_BASE, username);

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
        struct ProfileResponse {
            success: bool,
            user: Option<Profile>,
            error: Option<String>,
        }

        let resp: ProfileResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if !resp.success {
            return Err(resp.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        resp.user
            .ok_or_else(|| "User not found".to_string())
    }
}
