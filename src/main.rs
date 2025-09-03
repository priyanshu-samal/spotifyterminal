mod app;
mod auth;
mod config;

use anyhow::Result;
use rspotify::{
    prelude::*,
    scopes,
    AuthCodePkceSpotify, Credentials, OAuth, Token,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Please replace these with your own client ID and secret.
    // You can get them from the Spotify Developer Dashboard:
    // https://developer.spotify.com/dashboard/
    let client_id = "".to_string();
    let client_secret = "".to_string();

    let spotify = match config::load_tokens()? {
        Some(tokens) => {
            let token = Token {
                access_token: tokens.access_token,
                refresh_token: Some(tokens.refresh_token),
                ..Default::default()
            };
            let creds = Credentials::new(&client_id, &client_secret);
            let oauth = OAuth {
                redirect_uri: "http://127.0.0.1:8888/callback".to_string(),
                scopes: scopes!(
                    "user-read-private",
                    "user-read-email",
                    "playlist-read-private",
                    "playlist-read-collaborative",
                    "user-library-read",
                    "user-modify-playback-state",
                    "user-read-currently-playing",
                    "user-read-playback-state"
                ),
                ..Default::default()
            };

            let spotify = AuthCodePkceSpotify::new(creds, oauth);
            *spotify.token.lock().await.unwrap() = Some(token);
            match spotify.refresh_token().await {
                Ok(_) => println!("Refreshed token successfully!"),
                Err(e) => {
                    eprintln!("Failed to refresh token: {}. Re-authenticating...", e);
                    let new_tokens = auth::authenticate(client_id.clone(), client_secret.clone()).await?;
                    config::save_tokens(&new_tokens)?;
                    *spotify.token.lock().await.unwrap() = Some(Token {
                        access_token: new_tokens.access_token,
                        refresh_token: Some(new_tokens.refresh_token),
                        ..Default::default()
                    });
                    println!("Re-authenticated and saved new token successfully!");
                }
            }
            spotify
        }
        None => {
            let tokens = auth::authenticate(client_id.clone(), client_secret.clone()).await?;
            config::save_tokens(&tokens)?;
            let token = Token {
                access_token: tokens.access_token,
                refresh_token: Some(tokens.refresh_token),
                ..Default::default()
            };
            let creds = Credentials::new(&client_id, &client_secret);
            let oauth = OAuth {
                redirect_uri: "http://127.0.0.1:8888/callback".to_string(),
                scopes: scopes!(
                    "user-read-private",
                    "user-read-email",
                    "playlist-read-private",
                    "playlist-read-collaborative",
                    "user-library-read",
                    "user-modify-playback-state",
                    "user-read-currently-playing",
                    "user-read-playback-state"
                ),
                ..Default::default()
            };

            let spotify = AuthCodePkceSpotify::new(creds, oauth);
            *spotify.token.lock().await.unwrap() = Some(token);
            println!("Authenticated and saved token successfully!");
            spotify
        }
    };

    app::run_app(spotify).await?;

    Ok(())
}
