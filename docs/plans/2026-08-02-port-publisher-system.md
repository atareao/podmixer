# Port Publisher System from populatrs

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the publisher management system (X, Mastodon, Telegram, Matrix) from populatrs into podmixer, including CRUD management, test/control, minijinja templates, and publish logs with SSE streaming.

**Architecture:** Add a `Publisher` async trait with concrete implementations for each platform. A `PublisherManager` holds all configured publishers. A new `publishers` SQLite table replaces the key-value config pattern. A `publish_logs` table records every publish attempt with SSE streaming for real-time updates. The frontend gets a new Publishers page with dynamic forms per platform type.

**Tech Stack:** Rust Axum 0.8, SQLite (sqlx 0.8), minijinja 2.12, reqwest 0.12, React 19 + MUI v7 + TypeScript

## Global Constraints

- All API routes under `/api/v1/` prefix
- Use `ApiResponse` wrapper for all responses
- Follow existing patterns: class components in frontend, `Param` key-value for simple config
- New tables use sqlx migrations in `back/migrations/`
- SSE endpoint at `/api/v1/publishers/logs/stream`
- Frontend uses MUI v7 components with `sx` prop styling
- Dark mode default

---

### Task 1: Database migrations for publishers and logs

**Files:**
- Create: `back/migrations/20260802000001_publishers.up.sql`
- Create: `back/migrations/20260802000001_publishers.down.sql`
- Create: `back/migrations/20260802000002_publish_logs.up.sql`
- Create: `back/migrations/20260802000002_publish_logs.down.sql`

**Interfaces:**
- Consumes: existing sqlx migration infrastructure in `main.rs`
- Produces: `publishers` and `publish_logs` tables

- [ ] **Step 1: Create publishers migration**

```sql
-- 20260802000001_publishers.up.sql
CREATE TABLE IF NOT EXISTS publishers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    publisher_type TEXT NOT NULL,
    config TEXT NOT NULL DEFAULT '{}',
    template TEXT NOT NULL DEFAULT '',
    active INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

```sql
-- 20260802000001_publishers.down.sql
DROP TABLE IF EXISTS publishers;
```

- [ ] **Step 2: Create publish_logs migration**

```sql
-- 20260802000002_publish_logs.up.sql
CREATE TABLE IF NOT EXISTS publish_logs (
    id TEXT PRIMARY KEY,
    publisher_id TEXT NOT NULL,
    publisher_name TEXT NOT NULL,
    publisher_type TEXT NOT NULL,
    episode_title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    message TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (publisher_id) REFERENCES publishers(id)
);

CREATE INDEX IF NOT EXISTS idx_publish_logs_publisher_id ON publish_logs(publisher_id);
CREATE INDEX IF NOT EXISTS idx_publish_logs_created_at ON publish_logs(created_at);
```

```sql
-- 20260802000002_publish_logs.down.sql
DROP TABLE IF EXISTS publish_logs;
```

- [ ] **Step 3: Commit**

```bash
git add back/migrations/
git commit -m "🔧 chore: add publishers and publish_logs migrations"
```

---

### Task 2: Publisher trait and types

**Files:**
- Create: `back/src/models/publisher/mod.rs`
- Create: `back/src/models/publisher/types.rs`
- Create: `back/src/models/publisher/manager.rs`
- Modify: `back/src/models/mod.rs` (add publisher module)

**Interfaces:**
- Consumes: `AppState`, `SqlitePool`, `Error` type
- Produces: `Publisher` trait, `PublisherConfig` enum, `PublisherManager`, `PublishLog`

- [ ] **Step 1: Create publisher module directory and mod.rs**

```rust
// back/src/models/publisher/mod.rs
pub mod manager;
pub mod types;

pub use manager::PublisherManager;
pub use types::{PublishLog, Publisher, PublisherConfig, PublisherType};
```

- [ ] **Step 2: Create types.rs with Publisher trait and config types**

```rust
// back/src/models/publisher/types.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;

use super::super::Error;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PublisherType {
    Telegram,
    X,
    Mastodon,
    Matrix,
}

impl PublisherType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PublisherType::Telegram => "telegram",
            PublisherType::X => "x",
            PublisherType::Mastodon => "mastodon",
            PublisherType::Matrix => "matrix",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "telegram" => Some(PublisherType::Telegram),
            "x" => Some(PublisherType::X),
            "mastodon" => Some(PublisherType::Mastodon),
            "matrix" => Some(PublisherType::Matrix),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Publisher {
    pub id: String,
    pub name: String,
    pub publisher_type: PublisherType,
    pub config: serde_json::Value,
    pub template: String,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublishLog {
    pub id: String,
    pub publisher_id: String,
    pub publisher_name: String,
    pub publisher_type: String,
    pub episode_title: String,
    pub status: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublisherConfig {
    // Telegram
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
    pub message_thread_id: Option<String>,
    // X (Twitter)
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    // Mastodon
    pub server_url: Option<String>,
    pub access_token_mastodon: Option<String>,
    // Matrix
    pub homeserver_url: Option<String>,
    pub room_id: Option<String>,
    pub access_token_matrix: Option<String>,
}

#[async_trait]
pub trait PublisherImpl: Send + Sync {
    async fn publish(&self, title: &str, description: &str, url: &str) -> Result<String, Error>;
    fn publisher_type(&self) -> PublisherType;
    fn id(&self) -> &str;
}
```

- [ ] **Step 3: Create manager.rs**

```rust
// back/src/models/publisher/manager.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::super::Error;
use super::types::{Publisher, PublisherImpl, PublisherType, PublishLog};
use sqlx::sqlite::SqlitePool;

pub struct PublisherManager {
    pool: SqlitePool,
    publishers: RwLock<HashMap<String, Arc<dyn PublisherImpl>>>,
}

impl PublisherManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            publishers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn load_publishers(&self) -> Result<(), Error> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, bool, String, String)>(
            "SELECT id, name, publisher_type, config, template, active, created_at, updated_at FROM publishers WHERE active = 1"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut publishers = self.publishers.write().await;
        publishers.clear();

        for (id, name, ptype, config_json, template, active, created_at, updated_at) in rows {
            let ptype = PublisherType::from_str(&ptype);
            if ptype.is_none() { continue; }
            let config: serde_json::Value = serde_json::from_str(&config_json).unwrap_or_default();
            let publisher = Publisher {
                id, name,
                publisher_type: ptype.unwrap(),
                config, template,
                active, created_at, updated_at,
            };
            // Factory would create the impl here
        }
        Ok(())
    }

    pub async fn get_publishers_db(&self) -> Result<Vec<Publisher>, Error> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, bool, String, String)>(
            "SELECT id, name, publisher_type, config, template, active, created_at, updated_at FROM publishers ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, name, ptype, config, template, active, created_at, updated_at)| {
            Publisher {
                id, name,
                publisher_type: PublisherType::from_str(&ptype).unwrap_or(PublisherType::Telegram),
                config: serde_json::from_str(&config).unwrap_or_default(),
                template, active, created_at, updated_at,
            }
        }).collect())
    }

    pub async fn create_publisher(&self, publisher: &Publisher) -> Result<Publisher, Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let config_str = serde_json::to_string(&publisher.config)?;
        sqlx::query(
            "INSERT INTO publishers (id, name, publisher_type, config, template, active) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&publisher.name)
        .bind(publisher.publisher_type.as_str())
        .bind(&config_str)
        .bind(&publisher.template)
        .bind(publisher.active)
        .execute(&self.pool)
        .await?;

        self.get_publisher(&id).await
    }

    pub async fn get_publisher(&self, id: &str) -> Result<Publisher, Error> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, bool, String, String)>(
            "SELECT id, name, publisher_type, config, template, active, created_at, updated_at FROM publishers WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Publisher {
            id: row.0, name: row.1,
            publisher_type: PublisherType::from_str(&row.2).unwrap_or(PublisherType::Telegram),
            config: serde_json::from_str(&row.3).unwrap_or_default(),
            template: row.4, active: row.5,
            created_at: row.6, updated_at: row.7,
        })
    }

    pub async fn update_publisher(&self, id: &str, publisher: &Publisher) -> Result<Publisher, Error> {
        let config_str = serde_json::to_string(&publisher.config)?;
        sqlx::query(
            "UPDATE publishers SET name = ?, publisher_type = ?, config = ?, template = ?, active = ?, updated_at = datetime('now') WHERE id = ?"
        )
        .bind(&publisher.name)
        .bind(publisher.publisher_type.as_str())
        .bind(&config_str)
        .bind(&publisher.template)
        .bind(publisher.active)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_publisher(id).await
    }

    pub async fn delete_publisher(&self, id: &str) -> Result<(), Error> {
        sqlx::query("DELETE FROM publishers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn toggle_publisher(&self, id: &str) -> Result<Publisher, Error> {
        sqlx::query("UPDATE publishers SET active = NOT active, updated_at = datetime('now') WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        self.get_publisher(id).await
    }

    pub async fn add_log(&self, log: &PublishLog) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO publish_logs (id, publisher_id, publisher_name, publisher_type, episode_title, status, message) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&log.id)
        .bind(&log.publisher_id)
        .bind(&log.publisher_name)
        .bind(&log.publisher_type)
        .bind(&log.episode_title)
        .bind(&log.status)
        .bind(&log.message)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_logs(&self, limit: i64, offset: i64) -> Result<Vec<PublishLog>, Error> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, String, String)>(
            "SELECT id, publisher_id, publisher_name, publisher_type, episode_title, status, message, created_at FROM publish_logs ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| PublishLog {
            id: r.0, publisher_id: r.1, publisher_name: r.2,
            publisher_type: r.3, episode_title: r.4,
            status: r.5, message: r.6, created_at: r.7,
        }).collect())
    }

    pub async fn get_logs_by_publisher(&self, publisher_id: &str, limit: i64) -> Result<Vec<PublishLog>, Error> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, String, String)>(
            "SELECT id, publisher_id, publisher_name, publisher_type, episode_title, status, message, created_at FROM publish_logs WHERE publisher_id = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(publisher_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| PublishLog {
            id: r.0, publisher_id: r.1, publisher_name: r.2,
            publisher_type: r.3, episode_title: r.4,
            status: r.5, message: r.6, created_at: r.7,
        }).collect())
    }
}
```

- [ ] **Step 4: Update models/mod.rs**

Add to `back/src/models/mod.rs`:
```rust
pub mod publisher;
// Add to pub use:
pub use publisher::{Publisher, PublisherManager, PublishLog, PublisherType, PublisherConfig};
```

- [ ] **Step 5: Add uuid dependency to Cargo.toml**

```toml
uuid = { version = "1.16", features = ["v4"] }
```

- [ ] **Step 6: Commit**

```bash
git add back/src/models/publisher/ back/src/models/mod.rs back/Cargo.toml back/Cargo.lock
git commit -m "✨ feat: add Publisher trait, types, and manager"
```

---

### Task 3: Publisher implementations (Telegram, X, Mastodon, Matrix)

**Files:**
- Create: `back/src/models/publisher/telegram.rs`
- Create: `back/src/models/publisher/x.rs`
- Create: `back/src/models/publisher/mastodon.rs`
- Create: `back/src/models/publisher/matrix.rs`
- Modify: `back/src/models/publisher/mod.rs`
- Modify: `back/src/models/publisher/manager.rs` (add factory function)

**Interfaces:**
- Consumes: `PublisherImpl` trait, `PublisherConfig`
- Produces: `TelegramPublisher`, `XPublisher`, `MastodonPublisher`, `MatrixPublisher`

- [ ] **Step 1: Create Telegram publisher**

```rust
// back/src/models/publisher/telegram.rs
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use super::super::Error;
use super::types::{PublisherImpl, PublisherType};

pub struct TelegramPublisher {
    id: String,
    bot_token: String,
    chat_id: String,
    message_thread_id: String,
}

impl TelegramPublisher {
    pub fn new(id: String, config: &Value) -> Option<Self> {
        Some(Self {
            id,
            bot_token: config.get("bot_token")?.as_str()?.to_string(),
            chat_id: config.get("chat_id")?.as_str()?.to_string(),
            message_thread_id: config.get("message_thread_id")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
        })
    }
}

#[async_trait]
impl PublisherImpl for TelegramPublisher {
    async fn publish(&self, title: &str, description: &str, url: &str) -> Result<String, Error> {
        let message = format!(
            "<b>{}</b>\n\n{}",
            title,
            description
        );
        let api_url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let params = vec![
            ("chat_id", self.chat_id.as_str()),
            ("message_thread_id", self.message_thread_id.as_str()),
            ("text", &message),
            ("parse_mode", "HTML"),
        ];
        let resp = Client::new()
            .post(&api_url)
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(resp)
    }

    fn publisher_type(&self) -> PublisherType {
        PublisherType::Telegram
    }

    fn id(&self) -> &str {
        &self.id
    }
}
```

- [ ] **Step 2: Create X (Twitter) publisher**

```rust
// back/src/models/publisher/x.rs
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use super::super::Error;
use super::types::{PublisherImpl, PublisherType};

pub struct XPublisher {
    id: String,
    client_id: String,
    client_secret: String,
    access_token: String,
    refresh_token: String,
}

impl XPublisher {
    pub fn new(id: String, config: &Value) -> Option<Self> {
        Some(Self {
            id,
            client_id: config.get("client_id")?.as_str()?.to_string(),
            client_secret: config.get("client_secret")?.as_str()?.to_string(),
            access_token: config.get("access_token")?.as_str()?.to_string(),
            refresh_token: config.get("refresh_token")?.as_str()?.to_string(),
        })
    }

    pub async fn refresh_access_token(&mut self) -> Result<(), Error> {
        let url = "https://api.twitter.com/2/oauth2/token";
        let params = [
            ("refresh_token", self.refresh_token.as_str()),
            ("grant_type", "refresh_token"),
            ("client_id", self.client_id.as_str()),
        ];
        let data: Value = Client::new()
            .post(url)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        self.access_token = data["access_token"].as_str().unwrap_or("").to_string();
        self.refresh_token = data["refresh_token"].as_str().unwrap_or("").to_string();
        Ok(())
    }
}

#[async_trait]
impl PublisherImpl for XPublisher {
    async fn publish(&self, title: &str, _description: &str, url: &str) -> Result<String, Error> {
        let message = format!("{} {}", title, url);
        let api_url = "https://api.twitter.com/2/tweets";
        let body = json!({ "text": message });
        let resp = Client::new()
            .post(api_url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(resp)
    }

    fn publisher_type(&self) -> PublisherType {
        PublisherType::X
    }

    fn id(&self) -> &str {
        &self.id
    }
}
```

- [ ] **Step 3: Create Mastodon publisher**

```rust
// back/src/models/publisher/mastodon.rs
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use super::super::Error;
use super::types::{PublisherImpl, PublisherType};

pub struct MastodonPublisher {
    id: String,
    server_url: String,
    access_token: String,
}

impl MastodonPublisher {
    pub fn new(id: String, config: &Value) -> Option<Self> {
        Some(Self {
            id,
            server_url: config.get("server_url")?.as_str()?.to_string(),
            access_token: config.get("access_token")?.as_str()?.to_string(),
        })
    }
}

#[async_trait]
impl PublisherImpl for MastodonPublisher {
    async fn publish(&self, title: &str, description: &str, url: &str) -> Result<String, Error> {
        let status = format!("{}\n\n{}\n{}", title, description, url);
        let api_url = format!("{}/api/v1/statuses", self.server_url.trim_end_matches('/'));
        let params = [("status", status.as_str())];
        let resp = Client::new()
            .post(&api_url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(resp)
    }

    fn publisher_type(&self) -> PublisherType {
        PublisherType::Mastodon
    }

    fn id(&self) -> &str {
        &self.id
    }
}
```

- [ ] **Step 4: Create Matrix publisher**

```rust
// back/src/models/publisher/matrix.rs
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use super::super::Error;
use super::types::{PublisherImpl, PublisherType};

pub struct MatrixPublisher {
    id: String,
    homeserver_url: String,
    room_id: String,
    access_token: String,
}

impl MatrixPublisher {
    pub fn new(id: String, config: &Value) -> Option<Self> {
        Some(Self {
            id,
            homeserver_url: config.get("homeserver_url")?.as_str()?.to_string(),
            room_id: config.get("room_id")?.as_str()?.to_string(),
            access_token: config.get("access_token")?.as_str()?.to_string(),
        })
    }
}

#[async_trait]
impl PublisherImpl for MatrixPublisher {
    async fn publish(&self, title: &str, description: &str, url: &str) -> Result<String, Error> {
        let html = format!("<b>{}</b><br/>{}<br/><a href=\"{}\">{}</a>", title, description, url, url);
        let body = json!({
            "msgtype": "m.text",
            "format": "org.matrix.custom.html",
            "body": format!("{}\n{}\n{}", title, description, url),
            "formatted_body": html,
        });
        let txn_id = uuid::Uuid::new_v4().to_string();
        let api_url = format!(
            "{}/_matrix/client/v3/rooms/{}/send/m.room.message/{}",
            self.homeserver_url.trim_end_matches('/'),
            self.room_id,
            txn_id
        );
        let resp = Client::new()
            .put(&api_url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(resp)
    }

    fn publisher_type(&self) -> PublisherType {
        PublisherType::Matrix
    }

    fn id(&self) -> &str {
        &self.id
    }
}
```

- [ ] **Step 5: Update publisher/mod.rs**

```rust
// back/src/models/publisher/mod.rs
pub mod manager;
pub mod mastodon;
pub mod matrix;
pub mod telegram;
pub mod types;
pub mod x;

pub use manager::PublisherManager;
pub use types::{PublishLog, Publisher, PublisherConfig, PublisherImpl, PublisherType};
```

- [ ] **Step 6: Add factory function to manager.rs**

Add to `back/src/models/publisher/manager.rs`:
```rust
use super::telegram::TelegramPublisher;
use super::x::XPublisher;
use super::mastodon::MastodonPublisher;
use super::matrix::MatrixPublisher;

// Add method to PublisherManager:
pub fn create_publisher_impl(id: &str, publisher_type: &PublisherType, config: &serde_json::Value) -> Option<Arc<dyn PublisherImpl>> {
    match publisher_type {
        PublisherType::Telegram => {
            TelegramPublisher::new(id.to_string(), config)
                .map(|p| Arc::new(p) as Arc<dyn PublisherImpl>)
        }
        PublisherType::X => {
            XPublisher::new(id.to_string(), config)
                .map(|p| Arc::new(p) as Arc<dyn PublisherImpl>)
        }
        PublisherType::Mastodon => {
            MastodonPublisher::new(id.to_string(), config)
                .map(|p| Arc::new(p) as Arc<dyn PublisherImpl>)
        }
        PublisherType::Matrix => {
            MatrixPublisher::new(id.to_string(), config)
                .map(|p| Arc::new(p) as Arc<dyn PublisherImpl>)
        }
    }
}
```

- [ ] **Step 7: Commit**

```bash
git add back/src/models/publisher/
git commit -m "✨ feat: add Telegram, X, Mastodon, Matrix publisher implementations"
```

---

### Task 4: Template system with minijinja

**Files:**
- Create: `back/src/models/publisher/template.rs`
- Modify: `back/src/models/publisher/mod.rs`

**Interfaces:**
- Consumes: `minijinja::Environment`
- Produces: `TemplateRenderer` with custom filters

- [ ] **Step 1: Create template.rs**

```rust
// back/src/models/publisher/template.rs
use minijinja::{Environment, Value};
use serde::Serialize;

use super::super::Error;

#[derive(Debug, Serialize)]
pub struct TemplateContext {
    pub title: String,
    pub description: String,
    pub url: String,
}

pub struct TemplateRenderer;

impl TemplateRenderer {
    pub fn render(template_str: &str, ctx: &TemplateContext) -> Result<String, Error> {
        let mut env = Environment::new();
        env.add_filter("truncate", |value: String, length: usize| -> String {
            match value.char_indices().nth(length) {
                Some((idx, _)) => value[..idx].to_string(),
                None => value,
            }
        });
        env.add_filter("word_limit", |value: String, count: usize| -> String {
            value.split_whitespace()
                .take(count)
                .collect::<Vec<&str>>()
                .join(" ")
        });
        env.add_filter("strip_html", |value: String| -> String {
            let mut result = String::new();
            let mut in_tag = false;
            for c in value.chars() {
                match c {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    _ => if !in_tag { result.push(c); }
                }
            }
            result
        });
        env.add_template("tpl", template_str)?;
        let tmpl = env.get_template("tpl")?;
        let result = tmpl.render(minijinja::context!(
            title => ctx.title.as_str(),
            description => ctx.description.as_str(),
            url => ctx.url.as_str(),
        ))?;
        Ok(result)
    }
}
```

- [ ] **Step 2: Update publisher/mod.rs**

```rust
pub mod template;
pub use template::{TemplateContext, TemplateRenderer};
```

- [ ] **Step 3: Commit**

```bash
git add back/src/models/publisher/template.rs back/src/models/publisher/mod.rs
git commit -m "✨ feat: add minijinja template system with custom filters"
```

---

### Task 5: SSE broadcast layer for publish logs

**Files:**
- Create: `back/src/models/publisher/sse.rs`
- Modify: `back/src/models/publisher/mod.rs`

**Interfaces:**
- Consumes: `PublishLog`
- Produces: `SseBroadcaster` with `broadcast()` and `subscribe()` methods

- [ ] **Step 1: Create sse.rs**

```rust
// back/src/models/publisher/sse.rs
use std::sync::Arc;
use tokio::sync::broadcast;
use serde_json::json;

use super::types::PublishLog;

#[derive(Clone)]
pub struct SseBroadcaster {
    tx: broadcast::Sender<String>,
}

impl SseBroadcaster {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn broadcast(&self, log: &PublishLog) {
        let data = json!({
            "id": log.id,
            "publisher_id": log.publisher_id,
            "publisher_name": log.publisher_name,
            "publisher_type": log.publisher_type,
            "episode_title": log.episode_title,
            "status": log.status,
            "message": log.message,
            "created_at": log.created_at,
        });
        let _ = self.tx.send(data.to_string());
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}
```

- [ ] **Step 2: Update publisher/mod.rs**

```rust
pub mod sse;
pub use sse::SseBroadcaster;
```

- [ ] **Step 3: Add SseBroadcaster to AppState**

Modify `back/src/models/mod.rs`:
```rust
use super::publisher::SseBroadcaster;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub secret: String,
    pub sse_broadcaster: SseBroadcaster,
}
```

- [ ] **Step 4: Update main.rs AppState initialization**

In `back/src/main.rs`, update the AppState creation:
```rust
.with_state(Arc::new(AppState {
    pool: pool.clone(),
    secret,
    sse_broadcaster: SseBroadcaster::new(),
}));
```

- [ ] **Step 5: Commit**

```bash
git add back/src/models/publisher/sse.rs back/src/models/publisher/mod.rs back/src/models/mod.rs back/src/main.rs
git commit -m "✨ feat: add SSE broadcast layer for publish logs"
```

---

### Task 6: Publishers API routes

**Files:**
- Create: `back/src/http/publishers.rs`
- Modify: `back/src/http/mod.rs`
- Modify: `back/src/main.rs` (add router)

**Interfaces:**
- Consumes: `PublisherManager`, `SseBroadcaster`, `AppState`
- Produces: REST API endpoints for publishers CRUD, test, toggle, logs, SSE stream

- [ ] **Step 1: Create publishers.rs**

```rust
// back/src/http/publishers.rs
use std::sync::Arc;
use std::convert::Infallible;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing, Json, Router,
};
use futures::stream::Stream;
use serde::Deserialize;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, error};

use crate::models::{
    publisher::{
        manager::PublisherManager,
        template::{TemplateContext, TemplateRenderer},
        types::{PublishLog, Publisher, PublisherType},
        SseBroadcaster,
    },
    ApiResponse, AppState, Data, Error,
};

#[derive(Deserialize)]
pub struct LogsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub fn publishers_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", routing::get(list_publishers))
        .route("/", routing::post(create_publisher))
        .route("/{id}", routing::get(get_publisher))
        .route("/{id}", routing::patch(update_publisher))
        .route("/{id}", routing::delete(delete_publisher))
        .route("/{id}/toggle", routing::post(toggle_publisher))
        .route("/{id}/test", routing::post(test_publisher))
        .route("/logs", routing::get(get_logs))
        .route("/logs/stream", routing::get(stream_logs))
}

pub async fn list_publishers(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    match manager.get_publishers_db().await {
        Ok(publishers) => ApiResponse::new(StatusCode::OK, "Publishers list", Data::Many(serde_json::to_value(publishers).unwrap())),
        Err(e) => {
            error!("Error listing publishers: {:?}", e);
            ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, "Error listing publishers", Data::None)
        }
    }
}

pub async fn create_publisher(
    State(app_state): State<Arc<AppState>>,
    Json(publisher): Json<Publisher>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    match manager.create_publisher(&publisher).await {
        Ok(p) => ApiResponse::new(StatusCode::CREATED, "Publisher created", Data::One(serde_json::to_value(p).unwrap())),
        Err(e) => {
            error!("Error creating publisher: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error creating publisher", Data::None)
        }
    }
}

pub async fn get_publisher(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    match manager.get_publisher(&id).await {
        Ok(p) => ApiResponse::new(StatusCode::OK, "Publisher found", Data::One(serde_json::to_value(p).unwrap())),
        Err(e) => {
            error!("Error getting publisher: {:?}", e);
            ApiResponse::new(StatusCode::NOT_FOUND, "Publisher not found", Data::None)
        }
    }
}

pub async fn update_publisher(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(publisher): Json<Publisher>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    match manager.update_publisher(&id, &publisher).await {
        Ok(p) => ApiResponse::new(StatusCode::OK, "Publisher updated", Data::One(serde_json::to_value(p).unwrap())),
        Err(e) => {
            error!("Error updating publisher: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error updating publisher", Data::None)
        }
    }
}

pub async fn delete_publisher(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    match manager.delete_publisher(&id).await {
        Ok(_) => ApiResponse::new(StatusCode::OK, "Publisher deleted", Data::None),
        Err(e) => {
            error!("Error deleting publisher: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error deleting publisher", Data::None)
        }
    }
}

pub async fn toggle_publisher(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    match manager.toggle_publisher(&id).await {
        Ok(p) => ApiResponse::new(StatusCode::OK, "Publisher toggled", Data::One(serde_json::to_value(p).unwrap())),
        Err(e) => {
            error!("Error toggling publisher: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error toggling publisher", Data::None)
        }
    }
}

pub async fn test_publisher(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    let publisher = match manager.get_publisher(&id).await {
        Ok(p) => p,
        Err(e) => {
            error!("Publisher not found: {:?}", e);
            return ApiResponse::new(StatusCode::NOT_FOUND, "Publisher not found", Data::None);
        }
    };

    let ptype = publisher.publisher_type.clone();
    let impl_instance = PublisherManager::create_publisher_impl(&id, &ptype, &publisher.config);
    if impl_instance.is_none() {
        return ApiResponse::new(StatusCode::BAD_REQUEST, "Invalid publisher config", Data::None);
    }
    let impl_instance = impl_instance.unwrap();

    let ctx = TemplateContext {
        title: "Test publication".to_string(),
        description: "This is a test message from Podmixer".to_string(),
        url: "https://podmixer.example.com".to_string(),
    };

    let message = if publisher.template.is_empty() {
        format!("{} - {} - {}", ctx.title, ctx.description, ctx.url)
    } else {
        match TemplateRenderer::render(&publisher.template, &ctx) {
            Ok(m) => m,
            Err(e) => return ApiResponse::new(StatusCode::BAD_REQUEST, &format!("Template error: {}", e), Data::None),
        }
    };

    let log_id = uuid::Uuid::new_v4().to_string();
    let log = PublishLog {
        id: log_id.clone(),
        publisher_id: publisher.id.clone(),
        publisher_name: publisher.name.clone(),
        publisher_type: publisher.publisher_type.as_str().to_string(),
        episode_title: "Test".to_string(),
        status: "sending".to_string(),
        message: String::new(),
        created_at: String::new(),
    };
    let _ = manager.add_log(&log).await;

    match impl_instance.publish(&ctx.title, &ctx.description, &ctx.url).await {
        Ok(response) => {
            let success_log = PublishLog {
                id: log_id,
                publisher_id: publisher.id,
                publisher_name: publisher.name,
                publisher_type: publisher.publisher_type.as_str().to_string(),
                episode_title: "Test".to_string(),
                status: "success".to_string(),
                message: "Test published successfully".to_string(),
                created_at: String::new(),
            };
            let _ = manager.add_log(&success_log).await;
            app_state.sse_broadcaster.broadcast(&success_log);
            ApiResponse::new(StatusCode::OK, "Test published", Data::One(serde_json::json!({"response": response})))
        }
        Err(e) => {
            let error_log = PublishLog {
                id: log_id,
                publisher_id: publisher.id,
                publisher_name: publisher.name,
                publisher_type: publisher.publisher_type.as_str().to_string(),
                episode_title: "Test".to_string(),
                status: "error".to_string(),
                message: format!("{}", e),
                created_at: String::new(),
            };
            let _ = manager.add_log(&error_log).await;
            app_state.sse_broadcaster.broadcast(&error_log);
            ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Publish error: {}", e), Data::None)
        }
    }
}

pub async fn get_logs(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<LogsQuery>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    match manager.get_logs(limit, offset).await {
        Ok(logs) => ApiResponse::new(StatusCode::OK, "Logs retrieved", Data::Many(serde_json::to_value(logs).unwrap())),
        Err(e) => {
            error!("Error getting logs: {:?}", e);
            ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, "Error getting logs", Data::None)
        }
    }
}

pub async fn stream_logs(
    State(app_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = app_state.sse_broadcaster.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| {
        match result {
            Ok(data) => Some(Ok(Event::default().data(data))),
            Err(_) => None,
        }
    });
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}
```

- [ ] **Step 2: Update http/mod.rs**

```rust
mod publishers;
pub use publishers::publishers_router;
```

- [ ] **Step 3: Add route to main.rs**

In `back/src/main.rs`, add to `api_routes`:
```rust
.nest("/publishers", publishers_router())
```

And add the import:
```rust
use http::publishers_router;
```

- [ ] **Step 4: Add dependencies to Cargo.toml**

```toml
tokio-stream = "0.1"
futures = "0.3.31"  # already exists, ensure it's there
```

- [ ] **Step 5: Commit**

```bash
git add back/src/http/publishers.rs back/src/http/mod.rs back/src/main.rs back/Cargo.toml
git commit -m "✨ feat: add publishers API routes with CRUD, test, toggle, logs, SSE"
```

---

### Task 7: Frontend publishers page

**Files:**
- Create: `front/src/pages/publishers_page.tsx`
- Create: `front/src/components/publishers/publisher_list.tsx`
- Create: `front/src/components/publishers/publisher_form.tsx`
- Create: `front/src/components/publishers/publisher_logs.tsx`
- Create: `front/src/components/publishers/publisher_test.tsx`
- Create: `front/src/models/publisher.tsx`
- Modify: `front/src/App.tsx` (add route)
- Modify: `front/src/components/nav_bar.tsx` (add nav link)

**Interfaces:**
- Consumes: API endpoints at `/api/v1/publishers/*`
- Produces: Publishers management UI with CRUD, test, logs

- [ ] **Step 1: Create publisher model**

```tsx
// front/src/models/publisher.tsx
export interface Publisher {
    id: string;
    name: string;
    publisher_type: 'telegram' | 'x' | 'mastodon' | 'matrix';
    config: PublisherConfig;
    template: string;
    active: boolean;
    created_at: string;
    updated_at: string;
}

export interface PublisherConfig {
    // Telegram
    bot_token?: string;
    chat_id?: string;
    message_thread_id?: string;
    // X (Twitter)
    client_id?: string;
    client_secret?: string;
    access_token?: string;
    refresh_token?: string;
    // Mastodon
    server_url?: string;
    access_token_mastodon?: string;
    // Matrix
    homeserver_url?: string;
    room_id?: string;
    access_token_matrix?: string;
}

export interface PublishLog {
    id: string;
    publisher_id: string;
    publisher_name: string;
    publisher_type: string;
    episode_title: string;
    status: string;
    message: string;
    created_at: string;
}

export interface ApiResponse<T> {
    status: number;
    message: string;
    data: T | T[] | null;
}
```

- [ ] **Step 2: Create publisher_list component**

```tsx
// front/src/components/publishers/publisher_list.tsx
import React from 'react';
import { DataGrid, GridColDef } from '@mui/x-data-grid';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import { Publisher } from '../../models/publisher';
import { BASE_URL } from '../../constants';

interface PublisherListProps {
    publishers: Publisher[];
    onEdit: (publisher: Publisher) => void;
    onRefresh: () => void;
    onTest: (id: string) => void;
    onToggle: (id: string) => void;
    onDelete: (id: string) => void;
}

export default class PublisherList extends React.Component<PublisherListProps> {
    columns: GridColDef[] = [
        { field: 'name', headerName: 'Name', flex: 1 },
        { field: 'publisher_type', headerName: 'Type', width: 120 },
        {
            field: 'active',
            headerName: 'Active',
            width: 100,
            renderCell: (params) => (
                <Switch
                    checked={params.value}
                    onChange={() => this.props.onToggle(params.row.id)}
                />
            ),
        },
        {
            field: 'actions',
            headerName: 'Actions',
            width: 250,
            renderCell: (params) => (
                <Stack direction="row" spacing={1}>
                    <Button size="small" onClick={() => this.props.onEdit(params.row)}>Edit</Button>
                    <Button size="small" color="success" onClick={() => this.props.onTest(params.row.id)}>Test</Button>
                    <Button size="small" color="error" onClick={() => this.props.onDelete(params.row.id)}>Delete</Button>
                </Stack>
            ),
        },
    ];

    render() {
        return (
            <div style={{ height: 400, width: '100%' }}>
                <DataGrid
                    rows={this.props.publishers}
                    columns={this.columns}
                    getRowId={(row) => row.id}
                    disableRowSelectionOnClick
                />
            </div>
        );
    }
}
```

- [ ] **Step 3: Create publisher_form component**

```tsx
// front/src/components/publishers/publisher_form.tsx
import React from 'react';
import TextField from '@mui/material/TextField';
import Grid from '@mui/material/Grid';
import Button from '@mui/material/Button';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import InputLabel from '@mui/material/InputLabel';
import FormControl from '@mui/material/FormControl';
import Switch from '@mui/material/Switch';
import Typography from '@mui/material/Typography';
import { Publisher, PublisherConfig } from '../../models/publisher';

interface PublisherFormProps {
    publisher?: Publisher;
    onSave: (publisher: Partial<Publisher>) => void;
    onCancel: () => void;
}

interface PublisherFormState {
    name: string;
    publisher_type: string;
    template: string;
    active: boolean;
    config: Record<string, string>;
}

export default class PublisherForm extends React.Component<PublisherFormProps, PublisherFormState> {
    constructor(props: PublisherFormProps) {
        super(props);
        const p = props.publisher;
        this.state = {
            name: p?.name || '',
            publisher_type: p?.publisher_type || 'telegram',
            template: p?.template || '',
            active: p?.active || false,
            config: this.configToState(p?.config || {}),
        };
    }

    configToState(config: PublisherConfig): Record<string, string> {
        return {
            bot_token: config.bot_token || '',
            chat_id: config.chat_id || '',
            message_thread_id: config.message_thread_id || '',
            client_id: config.client_id || '',
            client_secret: config.client_secret || '',
            access_token: config.access_token || '',
            refresh_token: config.refresh_token || '',
            server_url: config.server_url || '',
            access_token_mastodon: config.access_token_mastodon || '',
            homeserver_url: config.homeserver_url || '',
            room_id: config.room_id || '',
            access_token_matrix: config.access_token_matrix || '',
        };
    }

    getConfigFields(): { key: string; label: string }[] {
        switch (this.state.publisher_type) {
            case 'telegram':
                return [
                    { key: 'bot_token', label: 'Bot Token' },
                    { key: 'chat_id', label: 'Chat ID' },
                    { key: 'message_thread_id', label: 'Message Thread ID' },
                ];
            case 'x':
                return [
                    { key: 'client_id', label: 'Client ID' },
                    { key: 'client_secret', label: 'Client Secret' },
                    { key: 'access_token', label: 'Access Token' },
                    { key: 'refresh_token', label: 'Refresh Token' },
                ];
            case 'mastodon':
                return [
                    { key: 'server_url', label: 'Server URL' },
                    { key: 'access_token_mastodon', label: 'Access Token' },
                ];
            case 'matrix':
                return [
                    { key: 'homeserver_url', label: 'Homeserver URL' },
                    { key: 'room_id', label: 'Room ID' },
                    { key: 'access_token_matrix', label: 'Access Token' },
                ];
            default:
                return [];
        }
    }

    handleSave = () => {
        const config: PublisherConfig = {};
        for (const field of this.getConfigFields()) {
            (config as any)[field.key] = this.state.config[field.key] || '';
        }
        this.props.onSave({
            name: this.state.name,
            publisher_type: this.state.publisher_type as any,
            template: this.state.template,
            active: this.state.active,
            config: config as any,
        });
    };

    render() {
        const fields = this.getConfigFields();
        return (
            <Grid container spacing={2}>
                <Grid size={12}>
                    <Typography variant="h6">
                        {this.props.publisher ? 'Edit Publisher' : 'New Publisher'}
                    </Typography>
                </Grid>
                <Grid size={6}>
                    <TextField
                        fullWidth label="Name" variant="outlined"
                        value={this.state.name}
                        onChange={(e) => this.setState({ name: e.target.value })}
                    />
                </Grid>
                <Grid size={6}>
                    <FormControl fullWidth>
                        <InputLabel>Type</InputLabel>
                        <Select
                            value={this.state.publisher_type}
                            label="Type"
                            onChange={(e) => this.setState({ publisher_type: e.target.value })}
                        >
                            <MenuItem value="telegram">Telegram</MenuItem>
                            <MenuItem value="x">X (Twitter)</MenuItem>
                            <MenuItem value="mastodon">Mastodon</MenuItem>
                            <MenuItem value="matrix">Matrix</MenuItem>
                        </Select>
                    </FormControl>
                </Grid>
                {fields.map((field) => (
                    <Grid size={6} key={field.key}>
                        <TextField
                            fullWidth label={field.label} variant="outlined"
                            value={this.state.config[field.key] || ''}
                            onChange={(e) => this.setState({
                                config: { ...this.state.config, [field.key]: e.target.value }
                            })}
                        />
                    </Grid>
                ))}
                <Grid size={12}>
                    <TextField
                        multiline minRows={4} fullWidth
                        label="Template (minijinja)" variant="outlined"
                        value={this.state.template}
                        onChange={(e) => this.setState({ template: e.target.value })}
                        helperText="Variables: {{ title }}, {{ description }}, {{ url }}. Filters: |truncate(n), |word_limit(n), |strip_html"
                    />
                </Grid>
                <Grid size={12}>
                    <Switch
                        checked={this.state.active}
                        onChange={(e) => this.setState({ active: e.target.checked })}
                    />
                    <Typography variant="button">Active</Typography>
                </Grid>
                <Grid size={12}>
                    <Stack direction="row" spacing={2}>
                        <Button variant="contained" onClick={this.handleSave}>Save</Button>
                        <Button variant="outlined" onClick={this.props.onCancel}>Cancel</Button>
                    </Stack>
                </Grid>
            </Grid>
        );
    }
}
```

- [ ] **Step 4: Create publisher_logs component**

```tsx
// front/src/components/publishers/publisher_logs.tsx
import React from 'react';
import { DataGrid, GridColDef } from '@mui/x-data-grid';
import Chip from '@mui/material/Chip';
import { PublishLog } from '../../models/publisher';

interface PublisherLogsProps {
    logs: PublishLog[];
}

export default class PublisherLogs extends React.Component<PublisherLogsProps> {
    columns: GridColDef[] = [
        { field: 'publisher_name', headerName: 'Publisher', width: 150 },
        { field: 'publisher_type', headerName: 'Type', width: 100 },
        { field: 'episode_title', headerName: 'Episode', flex: 1 },
        {
            field: 'status',
            headerName: 'Status',
            width: 120,
            renderCell: (params) => {
                const color = params.value === 'success' ? 'success' :
                    params.value === 'error' ? 'error' : 'warning';
                return <Chip label={params.value} color={color} size="small" />;
            },
        },
        { field: 'message', headerName: 'Message', flex: 1 },
        { field: 'created_at', headerName: 'Date', width: 180 },
    ];

    render() {
        return (
            <div style={{ height: 400, width: '100%' }}>
                <DataGrid
                    rows={this.props.logs}
                    columns={this.columns}
                    getRowId={(row) => row.id}
                    disableRowSelectionOnClick
                />
            </div>
        );
    }
}
```

- [ ] **Step 5: Create publishers_page**

```tsx
// front/src/pages/publishers_page.tsx
import React from 'react';
import Paper from '@mui/material/Paper';
import Box from '@mui/material/Box';
import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import DialogContent from '@mui/material/DialogContent';
import CustomTabPanel from '../components/custom_tab_panel';
import PublisherList from '../components/publishers/publisher_list';
import PublisherForm from '../components/publishers/publisher_form';
import PublisherLogs from '../components/publishers/publisher_logs';
import { Publisher, PublishLog } from '../models/publisher';
import { BASE_URL } from '../constants';

interface PublishersPageState {
    tabIndex: number;
    publishers: Publisher[];
    logs: PublishLog[];
    editingPublisher?: Publisher;
    showForm: boolean;
}

export default class PublishersPage extends React.Component<{}, PublishersPageState> {
    constructor(props: {}) {
        super(props);
        this.state = {
            tabIndex: 0,
            publishers: [],
            logs: [],
            showForm: false,
        };
    }

    componentDidMount() {
        this.loadPublishers();
        this.loadLogs();
        this.connectSse();
    }

    connectSse = () => {
        const eventSource = new EventSource(`${BASE_URL}/api/v1/publishers/logs/stream`);
        eventSource.onmessage = (event) => {
            try {
                const log = JSON.parse(event.data) as PublishLog;
                this.setState((prev) => ({
                    logs: [log, ...prev.logs].slice(0, 100),
                }));
            } catch (e) {
                console.error('SSE parse error:', e);
            }
        };
        eventSource.onerror = () => {
            console.error('SSE connection error');
            eventSource.close();
        };
    };

    loadPublishers = async () => {
        try {
            const resp = await fetch(`${BASE_URL}/api/v1/publishers/`);
            const json = await resp.json();
            if (resp.ok) {
                this.setState({ publishers: json.data || [] });
            }
        } catch (e) {
            console.error('Error loading publishers:', e);
        }
    };

    loadLogs = async () => {
        try {
            const resp = await fetch(`${BASE_URL}/api/v1/publishers/logs?limit=50`);
            const json = await resp.json();
            if (resp.ok) {
                this.setState({ logs: json.data || [] });
            }
        } catch (e) {
            console.error('Error loading logs:', e);
        }
    };

    handleSave = async (publisher: Partial<Publisher>) => {
        const isEdit = !!this.state.editingPublisher;
        const url = isEdit
            ? `${BASE_URL}/api/v1/publishers/${this.state.editingPublisher!.id}`
            : `${BASE_URL}/api/v1/publishers/`;
        const method = isEdit ? 'PATCH' : 'POST';

        try {
            const resp = await fetch(url, {
                method,
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(publisher),
            });
            if (resp.ok) {
                this.setState({ showForm: false, editingPublisher: undefined });
                this.loadPublishers();
            }
        } catch (e) {
            console.error('Error saving publisher:', e);
        }
    };

    handleTest = async (id: string) => {
        try {
            await fetch(`${BASE_URL}/api/v1/publishers/${id}/test`, { method: 'POST' });
        } catch (e) {
            console.error('Error testing publisher:', e);
        }
    };

    handleToggle = async (id: string) => {
        try {
            await fetch(`${BASE_URL}/api/v1/publishers/${id}/toggle`, { method: 'POST' });
            this.loadPublishers();
        } catch (e) {
            console.error('Error toggling publisher:', e);
        }
    };

    handleDelete = async (id: string) => {
        try {
            await fetch(`${BASE_URL}/api/v1/publishers/${id}`, { method: 'DELETE' });
            this.loadPublishers();
        } catch (e) {
            console.error('Error deleting publisher:', e);
        }
    };

    render() {
        return (
            <Paper sx={{ width: '100%', p: 2 }}>
                <h1>Publishers</h1>
                <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
                    <Tabs value={this.state.tabIndex} onChange={(_, v) => this.setState({ tabIndex: v })}>
                        <Tab label="Publishers" />
                        <Tab label="Logs" />
                    </Tabs>
                </Box>
                <CustomTabPanel value={this.state.tabIndex} index={0}>
                    <Button
                        variant="contained"
                        sx={{ mb: 2 }}
                        onClick={() => this.setState({ showForm: true, editingPublisher: undefined })}
                    >
                        Add Publisher
                    </Button>
                    <PublisherList
                        publishers={this.state.publishers}
                        onEdit={(p) => this.setState({ editingPublisher: p, showForm: true })}
                        onRefresh={this.loadPublishers}
                        onTest={this.handleTest}
                        onToggle={this.handleToggle}
                        onDelete={this.handleDelete}
                    />
                    <Dialog open={this.state.showForm} onClose={() => this.setState({ showForm: false })} maxWidth="md" fullWidth>
                        <DialogContent>
                            <PublisherForm
                                publisher={this.state.editingPublisher}
                                onSave={this.handleSave}
                                onCancel={() => this.setState({ showForm: false, editingPublisher: undefined })}
                            />
                        </DialogContent>
                    </Dialog>
                </CustomTabPanel>
                <CustomTabPanel value={this.state.tabIndex} index={1}>
                    <PublisherLogs logs={this.state.logs} />
                </CustomTabPanel>
            </Paper>
        );
    }
}
```

- [ ] **Step 6: Add route in App.tsx**

Add import and route:
```tsx
import PublishersPage from "./pages/publishers_page";
// Add inside ProtectedLayout:
<Route path="publishers" element={<PublishersPage />} />
```

- [ ] **Step 7: Add nav link in nav_bar.tsx**

Add to the navigation:
```tsx
<NavLink to="/publishers" className={...}>Publishers</NavLink>
```

- [ ] **Step 8: Commit**

```bash
git add front/src/pages/publishers_page.tsx front/src/components/publishers/ front/src/models/publisher.tsx front/src/App.tsx front/src/components/nav_bar.tsx
git commit -m "✨ feat: add publishers frontend page with CRUD, test, logs, SSE"
```

---

### Task 8: Integrate publishers into background worker

**Files:**
- Modify: `back/src/main.rs` (update `do_the_work` to use PublisherManager)

**Interfaces:**
- Consumes: `PublisherManager`, `TemplateRenderer`, `SseBroadcaster`
- Produces: Episodes published through all active publishers

- [ ] **Step 1: Update do_the_work function**

Replace the existing Telegram/Twitter publish logic in `do_the_work` with the new PublisherManager:

```rust
// In do_the_work, after generating new episodes:
if generate {
    let manager = PublisherManager::new(pool.clone());
    let publishers = manager.get_publishers_db().await?;
    let active_publishers: Vec<Publisher> = publishers.into_iter().filter(|p| p.active).collect();

    new_episodes.sort_by(|a, b| a.pub_date.cmp(&b.pub_date));
    for episode in new_episodes.as_slice() {
        let title = episode.title().unwrap_or("");
        let description = from_read(
            episode.description().unwrap_or("").as_bytes(),
            5000,
        ).unwrap_or_default();
        let url = episode.link().unwrap_or("");

        for publisher in &active_publishers {
            let ctx = TemplateContext {
                title: title.to_string(),
                description: description.clone(),
                url: url.to_string(),
            };

            let message = if publisher.template.is_empty() {
                format!("{} - {} - {}", title, description, url)
            } else {
                match TemplateRenderer::render(&publisher.template, &ctx) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Template error for {}: {}", publisher.name, e);
                        continue;
                    }
                }
            };

            let impl_instance = PublisherManager::create_publisher_impl(
                &publisher.id, &publisher.publisher_type, &publisher.config
            );

            if let Some(publisher_impl) = impl_instance {
                let log_id = uuid::Uuid::new_v4().to_string();
                let log = PublishLog {
                    id: log_id.clone(),
                    publisher_id: publisher.id.clone(),
                    publisher_name: publisher.name.clone(),
                    publisher_type: publisher.publisher_type.as_str().to_string(),
                    episode_title: title.to_string(),
                    status: "sending".to_string(),
                    message: String::new(),
                    created_at: String::new(),
                };
                let _ = manager.add_log(&log).await;

                match publisher_impl.publish(title, &description, url).await {
                    Ok(_) => {
                        let success_log = PublishLog {
                            id: log_id,
                            publisher_id: publisher.id.clone(),
                            publisher_name: publisher.name.clone(),
                            publisher_type: publisher.publisher_type.as_str().to_string(),
                            episode_title: title.to_string(),
                            status: "success".to_string(),
                            message: "Published successfully".to_string(),
                            created_at: String::new(),
                        };
                        let _ = manager.add_log(&success_log).await;
                        // Broadcast via SSE
                        // Need to pass SseBroadcaster - store in AppState or pass to do_the_work
                    }
                    Err(e) => {
                        let error_log = PublishLog {
                            id: log_id,
                            publisher_id: publisher.id.clone(),
                            publisher_name: publisher.name.clone(),
                            publisher_type: publisher.publisher_type.as_str().to_string(),
                            episode_title: title.to_string(),
                            status: "error".to_string(),
                            message: format!("{}", e),
                            created_at: String::new(),
                        };
                        let _ = manager.add_log(&error_log).await;
                        error!("Error publishing to {}: {}", publisher.name, e);
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
    // ... rest of feed generation
}
```

- [ ] **Step 2: Pass SseBroadcaster to do_the_work**

Update `do_the_work` signature to accept `SseBroadcaster`:
```rust
async fn do_the_work(pool: &SqlitePool, older_than: i32, sse: SseBroadcaster) -> Result<(), Error> {
```

Update the spawn call:
```rust
let sse = app_state.sse_broadcaster.clone();
tokio::spawn(async move {
    loop {
        match do_the_work(&pool2, older_than, sse.clone()).await {
            ...
        }
    }
});
```

- [ ] **Step 3: Commit**

```bash
git add back/src/main.rs
git commit -m "✨ feat: integrate PublisherManager into background worker"
```

---

### Task 9: Remove old Telegram/Twitter config and migrate data

**Files:**
- Modify: `back/src/http/config.rs` (remove telegram/twitter routes)
- Modify: `back/src/models/mod.rs` (remove old Telegram/Twitter exports)
- Keep: `back/src/models/telegram.rs` and `back/src/models/twitter.rs` (keep for backward compat during migration)

**Note:** This task is optional and can be done after the new system is verified working. The old config endpoints can remain for backward compatibility.

- [ ] **Step 1: Add migration to copy existing config to publishers table**

Create `back/migrations/20260802000003_migrate_publishers.up.sql`:
```sql
-- Migrate existing Telegram config to publishers table
INSERT OR IGNORE INTO publishers (id, name, publisher_type, config, template, active)
SELECT 
    'telegram-legacy',
    'Telegram (legacy)',
    'telegram',
    json_object(
        'bot_token', COALESCE((SELECT value FROM config WHERE key = 'telegram_token'), ''),
        'chat_id', COALESCE((SELECT value FROM config WHERE key = 'telegram_chat_id'), ''),
        'message_thread_id', COALESCE((SELECT value FROM config WHERE key = 'telegram_thread_id'), '0')
    ),
    COALESCE((SELECT value FROM config WHERE key = 'telegram_template'), ''),
    CASE WHEN (SELECT value FROM config WHERE key = 'telegram_active') = 'TRUE' THEN 1 ELSE 0 END
WHERE EXISTS (SELECT 1 FROM config WHERE key = 'telegram_token');

-- Migrate existing Twitter config to publishers table
INSERT OR IGNORE INTO publishers (id, name, publisher_type, config, template, active)
SELECT 
    'x-legacy',
    'X (legacy)',
    'x',
    json_object(
        'client_id', COALESCE((SELECT value FROM config WHERE key = 'twitter_client_id'), ''),
        'client_secret', COALESCE((SELECT value FROM config WHERE key = 'twitter_client_secret'), ''),
        'access_token', COALESCE((SELECT value FROM config WHERE key = 'twitter_access_token'), ''),
        'refresh_token', COALESCE((SELECT value FROM config WHERE key = 'twitter_refresh_token'), '')
    ),
    COALESCE((SELECT value FROM config WHERE key = 'twitter_template'), ''),
    CASE WHEN (SELECT value FROM config WHERE key = 'twitter_active') = 'TRUE' THEN 1 ELSE 0 END
WHERE EXISTS (SELECT 1 FROM config WHERE key = 'twitter_client_id');
```

- [ ] **Step 2: Commit**

```bash
git add back/migrations/20260802000003_migrate_publishers.up.sql
git commit -m "🔧 chore: add migration to copy existing config to publishers table"
```

---

### Task 10: Build verification and final commit

**Files:** All modified files

- [ ] **Step 1: Run cargo check**

```bash
cd back && cargo check 2>&1
```
Expected: Compilation successful

- [ ] **Step 2: Run cargo test**

```bash
cd back && cargo test 2>&1
```
Expected: 13 pass, 3 pre-existing failures

- [ ] **Step 3: Run frontend build**

```bash
cd front && pnpm run build 2>&1
```
Expected: TypeScript + Vite build successful

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "✅ chore: finalize publisher system port from populatrs"
```