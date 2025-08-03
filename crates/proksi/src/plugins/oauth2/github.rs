use std::borrow::Cow;

use anyhow::bail;
use serde::Deserialize;

use super::{provider::{OauthType, OauthUser, Provider}, HTTP_CLIENT};

/// Github `OAuth2` plugin
pub(super) struct GithubOauthService;

#[allow(dead_code)]
const GITHUB_API_URL: &str = "https://api.github.com";
const GITHUB_OAUTH_URL: &str = "https://github.com/login/oauth/authorize";
#[allow(dead_code)]
const GITHUB_OAUTH_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

impl GithubOauthService {
    /// Add a new method to create a Provider
    pub fn new(client_id: &str, client_secret: &str) -> Provider {
        Provider {
            typ: OauthType::Github,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        }
    }

    /// Get the OAuth callback URL
    /// This is used to redirect the user to the Github login page
    /// and then to the OAuth callback URL
    /// The state parameter is used to prevent CSRF attacks
    /// and to ensure that the callback is coming from the correct source
    ///
    /// user:email is the scope that is requested and you should update the app settings
    /// to add the scope to the app
    pub fn get_oauth_callback_url(client_id: &str, state: &str) -> String {
        let redirect_uri = "";

        format!("{GITHUB_OAUTH_URL}?client_id={client_id}&redirect_uri={redirect_uri}&state={state}&scope=user:email&response_type=code")
    }

    #[allow(dead_code)]
    pub async fn get_oauth_user(
        client_id: &str,
        client_secret: &str,
        code: &str,
    ) -> Result<OauthUser, anyhow::Error> {
        let token = Self::get_oauth_token(client_id, client_secret, code).await?;

        if token.access_token.is_none() {
            bail!("Failed to get access token from Github: {:?}", token.error);
        }

        let access_token = token.access_token.unwrap();

        let user = Self::get_user_info(&access_token).await?;
        let emails = Self::get_user_emails(&access_token).await?;

        let primary_email = emails.iter().find(|e| e.primary).unwrap();

        Ok(OauthUser {
            email: Cow::Owned(primary_email.email.to_string()),
            team_ids: vec![],
            organization_ids: vec![],
            usernames: vec![user.login.to_string()],
        })
    }

    /// Get the OAuth token
    /// Retrieves the token from the Github API to be used for further requests
    #[allow(dead_code)]
    async fn get_oauth_token(
        client_id: &str,
        client_secret: &str,
        code: &str,
    ) -> Result<GithubTokenResponse, anyhow::Error> {
        let response = HTTP_CLIENT
            .post(GITHUB_OAUTH_TOKEN_URL)
            .query(&[
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("code", code),
            ])
            .header(http::header::ACCEPT, "application/json")
            .send()
            .await?;

        let body = response.json::<GithubTokenResponse>().await?;
        Ok(body)
    }

    /// Get the user public profile information
    #[allow(dead_code)]
    async fn get_user_info(token: &str) -> Result<GithubUserResponse, anyhow::Error> {
        tracing::debug!("Getting user info from Github");

        let response = HTTP_CLIENT
            .get(format!("{GITHUB_API_URL}/user"))
            .bearer_auth(token)
            .header(http::header::USER_AGENT, "pingora/0.2.0")
            .send()
            .await?;

        let body = response.json::<GithubUserResponse>().await?;
        Ok(body)
    }

    /// Get the user emails in order to find the primary one
    #[allow(dead_code)]
    async fn get_user_emails(token: &str) -> Result<Vec<GithubEmailResponse>, anyhow::Error> {
        tracing::debug!("Getting user emails from Github");

        let response = HTTP_CLIENT
            .get(format!("{GITHUB_API_URL}/user/emails"))
            .bearer_auth(token)
            .header(http::header::USER_AGENT, "pingora/0.2.0")
            .send()
            .await?;

        let body = response.json::<Vec<GithubEmailResponse>>().await?;
        Ok(body)
    }
}

/// Response from `POST github.com/login/oauth/access_token`
/// Can be an error response (with an error property) or a success response
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct GithubTokenResponse {
    error: Option<Cow<'static, str>>,
    access_token: Option<Cow<'static, str>>,
}

/// Item Response from `api.github.com/user/emails`
/// `{ [ {"email":"user@example.com","primary":true,"verified":true} ]  }`
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct GithubEmailResponse {
    email: Cow<'static, str>,
    primary: bool,
}

/// Response from `api.github.com/user`
/// `{ "name": "John Doe", username: "johndoe" } }`
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct GithubUserResponse {
    login: Cow<'static, str>,
}
