use anyhow::Result;
use reqwest::Client;

pub struct Telegram {
    token:   String,
    chat_id: String,
    client:  Client,
}

impl Telegram {
    pub fn new(token: &str, chat_id: &str) -> Self {
        Self {
            token:   token.to_string(),
            chat_id: chat_id.to_string(),
            client:  Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    pub async fn send(&self, text: &str) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        self.client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id":    self.chat_id,
                "text":       text,
                "parse_mode": "HTML"
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn alert_down(&self, host: &str, details: &str) -> Result<()> {
        let msg = format!(
            "🚨 <b>GODO-VPS DOWN</b>\nHost: <code>{host}</code>\n{details}"
        );
        self.send(&msg).await
    }

    pub async fn alert_recovered(&self, host: &str) -> Result<()> {
        let msg = format!("✅ <b>GODO-VPS recovered</b>\nHost: <code>{host}</code> is healthy again.");
        self.send(&msg).await
    }
}

/// No-op notifier when Telegram is not configured.
pub enum Notifier {
    Telegram(Telegram),
    Silent,
}

impl Notifier {
    pub fn new(token: Option<&str>, chat_id: Option<&str>) -> Self {
        match (token, chat_id) {
            (Some(t), Some(c)) if !t.is_empty() && !c.is_empty() => {
                Self::Telegram(Telegram::new(t, c))
            }
            _ => Self::Silent,
        }
    }

    pub async fn alert_down(&self, host: &str, details: &str) {
        if let Self::Telegram(tg) = self {
            if let Err(e) = tg.alert_down(host, details).await {
                tracing::warn!("Telegram send failed: {e}");
            }
        }
    }

    pub async fn alert_recovered(&self, host: &str) {
        if let Self::Telegram(tg) = self {
            if let Err(e) = tg.alert_recovered(host).await {
                tracing::warn!("Telegram send failed: {e}");
            }
        }
    }
}
