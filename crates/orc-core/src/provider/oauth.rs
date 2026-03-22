use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::config::tokens_dir;

const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const AUTHORIZE_URL: &str = "https://claude.ai/oauth/authorize";
const TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
const SCOPES: &str = "org:create_api_key user:profile user:inference";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
}

pub fn load_tokens(provider_id: &str) -> Result<Option<OAuthTokens>> {
    let path = tokens_dir()?.join(format!("{provider_id}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let tokens: OAuthTokens = serde_json::from_str(&content)?;
    Ok(Some(tokens))
}

pub fn save_tokens(provider_id: &str, tokens: &OAuthTokens) -> Result<()> {
    let path = tokens_dir()?.join(format!("{provider_id}.json"));
    let content = serde_json::to_string_pretty(tokens)?;
    std::fs::write(&path, content)?;
    Ok(())
}

fn generate_code_verifier() -> String {
    let mut rng = rand::rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.random::<u8>()).collect();
    URL_SAFE_NO_PAD.encode(&bytes)
}

fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

pub async fn run_oauth_flow(provider_id: &str) -> Result<OAuthTokens> {
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);

    // 로컬 서버 시작
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://127.0.0.1:{port}/oauth/callback");

    // 브라우저에서 인증 페이지 열기
    let auth_url = format!(
        "{AUTHORIZE_URL}?response_type=code&client_id={CLIENT_ID}&redirect_uri={}&scope={}&code_challenge={code_challenge}&code_challenge_method=S256",
        urlencoded(&redirect_uri),
        urlencoded(SCOPES),
    );

    open::that(&auth_url).context("failed to open browser")?;

    // 콜백 대기
    let code = wait_for_callback(listener).await?;

    // 토큰 교환
    let tokens = exchange_code(&code, &code_verifier, &redirect_uri).await?;
    save_tokens(provider_id, &tokens)?;

    Ok(tokens)
}

async fn wait_for_callback(listener: tokio::net::TcpListener) -> Result<String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let (mut stream, _) = listener.accept().await?;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);

    // GET /oauth/callback?code=xxx HTTP/1.1 에서 code 추출
    let code = request
        .lines()
        .next()
        .and_then(|line| {
            let path = line.split_whitespace().nth(1)?;
            let query = path.split('?').nth(1)?;
            query.split('&').find_map(|param| {
                let (key, value) = param.split_once('=')?;
                if key == "code" {
                    Some(value.to_string())
                } else {
                    None
                }
            })
        })
        .context("no authorization code in callback")?;

    // 성공 응답
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h2>Authentication successful!</h2><p>You can close this tab and return to orc.</p></body></html>";
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    Ok(code)
}

async fn exchange_code(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<OAuthTokens> {
    let client = reqwest::Client::new();
    let response = client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", CLIENT_ID),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("code_verifier", code_verifier),
        ])
        .send()
        .await
        .context("failed to exchange authorization code")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("token exchange failed {status}: {text}");
    }

    let json: serde_json::Value = response.json().await?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let expires_at = json["expires_in"]
        .as_u64()
        .map(|secs| now + secs);

    Ok(OAuthTokens {
        access_token: json["access_token"]
            .as_str()
            .context("missing access_token")?
            .to_string(),
        refresh_token: json["refresh_token"].as_str().map(|s| s.to_string()),
        expires_at,
    })
}

pub async fn refresh_access_token(provider_id: &str, refresh_token: &str) -> Result<OAuthTokens> {
    let client = reqwest::Client::new();
    let response = client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", CLIENT_ID),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await
        .context("failed to refresh token")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("token refresh failed {status}: {text}");
    }

    let json: serde_json::Value = response.json().await?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let expires_at = json["expires_in"].as_u64().map(|secs| now + secs);

    let tokens = OAuthTokens {
        access_token: json["access_token"]
            .as_str()
            .context("missing access_token")?
            .to_string(),
        refresh_token: json["refresh_token"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| Some(refresh_token.to_string())),
        expires_at,
    };

    save_tokens(provider_id, &tokens)?;
    Ok(tokens)
}

impl OAuthTokens {
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now >= expires_at.saturating_sub(300) // 5분 여유
        } else {
            false
        }
    }
}

fn urlencoded(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                String::from(b as char)
            }
            _ => format!("%{:02X}", b),
        })
        .collect()
}
