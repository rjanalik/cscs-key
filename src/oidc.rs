//use std::fs::{File, metadata};
use std::io::Write;
use reqwest;
//use serde::{Serialize, Deserialize};
//use anyhow::{anyhow, bail, Context};
use anyhow::Context;
//use log::{info, debug};
use log::info;

use crate::config::Config;
//use crate::password_manager::PasswordManager;

use openidconnect::core::{CoreClient, CoreProviderMetadata, CoreResponseType};
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, IssuerUrl,
    PkceCodeChallenge, RedirectUrl, Scope,
    CsrfToken, Nonce,
    OAuth2TokenResponse,
    TokenResponse,
};
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use url::Url;

pub fn oidc_get_access_token(config: &Config) -> anyhow::Result<String> {
    info!("Get OIDC token");

    // In 4.x, we create a reusable reqwest client first
    let http_client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none()) // Recommended for OIDC security
        .build()?;

    let issuer_url = IssuerUrl::new(config.issuer_url.clone())?;

    // Discovery takes a reference to the client
    let provider_metadata = CoreProviderMetadata::discover(&issuer_url, &http_client)?;

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(config.pkce_client_id.clone()),
        None,
    )
    .set_redirect_uri(RedirectUrl::new("http://localhost:8765".to_string())?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random, // State/CSRF provider
            Nonce::new_random,     // Nonce provider
        )
        .add_scope(Scope::new("openid".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Open the browser!
    if let Err(e) = webbrowser::open(auth_url.as_str()) {
        eprintln!("Failed to open browser automatically: {}", e);
        println!("Browser window did not open automatically. Log in here :\n{}", auth_url);
    }

    // Simple listener
    let listener = TcpListener::bind("127.0.0.1:8765")?;
    let (mut stream, _) = listener.accept()?;
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    let redirect_url = request_line.split_whitespace().nth(1).unwrap_or("");
    let url = Url::parse(&format!("http://localhost:8765{}", redirect_url))?;

    // Check CSRF: Unlikely on localhost, but better be careful
    let returned_state = url.query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.into_owned())
        .context("No state found")?;
    if returned_state != *csrf_token.secret() {
        return Err(anyhow::anyhow!("CSRF detected! State mismatch."));
    }

    let code = url.query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.into_owned())
        .context("No code found")?;

    let response = "HTTP/1.1 200 OK\r\n\r\nAuthentication successful.\nYou can return to the terminal.";
    stream.write_all(response.as_bytes())?;

    // Pass the reference to the client here too
    let token_response = client
        .exchange_code(AuthorizationCode::new(code))?
        .set_pkce_verifier(pkce_verifier)
        .request(&http_client)?; // Look Ma, no http_client() helper!

    // Check nonce: Replay protection
    let id_token = token_response
        .id_token()
        .ok_or_else(|| anyhow::anyhow!("Server did not return an ID token"))?;
    let id_token_verifier = client.id_token_verifier();
    let claims = id_token.claims(&id_token_verifier, &nonce)?;
    println!("Verified identity for user: {}", claims.subject().as_str());

    Ok(token_response.access_token().secret().to_string())
}
