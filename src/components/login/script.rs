use oauth2::{
    basic::BasicClient, AuthType, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl, AuthorizationCode
};
use oauth2::reqwest::async_http_client;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use url::Url;

use crate::secrets;

const AUTH_URL: &str = "https://account.ely.by/oauth2/v1";
const TOKEN_URL: &str = "https://account.ely.by/api/oauth2/v1/token";

pub async fn login() -> anyhow::Result<String> {
    let client = BasicClient::new(
        ClientId::new(secrets::AUTH_CLIENT_ID.to_string()),
        Some(ClientSecret::new(secrets::AUTH_CLIENT_SECRET.to_string())),
        AuthUrl::new(AUTH_URL.to_string())?,
        Some(TokenUrl::new(TOKEN_URL.to_string())?),
    )
    .set_auth_type(AuthType::RequestBody)
    .set_redirect_uri(RedirectUrl::new(secrets::AUTH_REDIRECT_URI.to_string())?);

    // Generate the full authorization URL.
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("account_info".to_string()))
        .url();

    // Create a listener for the callback
    let listener = TcpListener::bind("localhost:23234").await?;
    
    // Open the browser
    if let Err(e) = webbrowser::open(auth_url.as_str()) {
        println!("Failed to open browser: {}", e);
    }

    // Wait for the callback
    println!("Waiting for callback...");
    let (mut stream, _) = listener.accept().await?;
    
    let mut buffer = [0; 2048];
    let n = stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..n]);

    // Parse the code and state from the query string
    // Request line example: GET /callback?code=...&state=... HTTP/1.1
    let url_part = request.lines().next().unwrap_or("").split_whitespace().nth(1).unwrap_or("");
    let url = Url::parse(&format!("http://127.0.0.1:23234{}", url_part))?;
    
    let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    
    let code = pairs.get("code").ok_or_else(|| anyhow::anyhow!("Missing code param"))?;
    let state = pairs.get("state").ok_or_else(|| anyhow::anyhow!("Missing state param"))?;

    if state != csrf_token.secret() {
        return Err(anyhow::anyhow!("Invalid state token"));
    }

    // Respond to the browser
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Login Successful!</h1><p>You can close this window now.</p><script>window.close()</script></body></html>";
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    // Exchange the code with a token.
    let token_result = client
        .exchange_code(AuthorizationCode::new(code.clone()))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            println!("Token exchange error: {:?}", e);
            anyhow::anyhow!("Token exchange failed: {:?}", e)
        })?;

    Ok(token_result.access_token().secret().to_string())
}

#[derive(serde::Deserialize, Debug)]
struct UserInfoResponse {
    id: u32,
    uuid: String,
    username: String,
    email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: u32,
    pub uuid: String,
    pub username: String,
    pub email: Option<String>,
    pub skin_url: String,
}

pub async fn fetch_user_info(token: &str) -> anyhow::Result<UserInfo> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://account.ely.by/api/account/v1/info")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to fetch user info: {}", response.status()));
    }

    let api_response = response.json::<UserInfoResponse>().await?;
    
    Ok(UserInfo {
        id: api_response.id,
        uuid: api_response.uuid,
        skin_url: format!("http://skinsystem.ely.by/skins/{}.png", api_response.username),
        username: api_response.username,
        email: api_response.email,
    })
}