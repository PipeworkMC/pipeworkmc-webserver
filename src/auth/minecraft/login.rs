use crate::util::dotenv;
use core::fmt;
use std::borrow::Cow;
use surf::{ Client, Body };
use urlencoding::encode as urlencode;
use serde::Serialize as Ser;
use serde::Deserialize as Deser;


const MICROSOFT_AZURE_SCOPE : &str = "XboxLive.signin offline_access";


pub fn build_microsoft_access_code_url(state : Option<&str>) -> String {
    let client_id    = dotenv::var("MICROSOFT_AZURE_CLIENT_ID");
    let redirect_uri = urlencode(dotenv::var("MICROSOFT_AZURE_REDIRECT_URI"));
    let state        = if let Some(state) = state { Cow::Owned(format!("&state={}", urlencode(state))) } else { Cow::Borrowed("") };
    let scope        = urlencode(MICROSOFT_AZURE_SCOPE);
    format!("https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize?client_id={client_id}&response_type=code&redirect_uri={redirect_uri}&scope={scope}{state}&prompt=select_account")
}


pub async fn exchange_microsoft_token(
    client         : &Client,
    microsoft_code : &str
) -> surf::Result<MicrosoftAccessToken> {
    let request = client.post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "application/json")
        .body(Body::from_form(&MicrosoftTokenQuery {
            client_id     : dotenv::var("MICROSOFT_AZURE_CLIENT_ID"),
            scope         : MICROSOFT_AZURE_SCOPE,
            code          : microsoft_code,
            redirect_uri  : dotenv::var("MICROSOFT_AZURE_REDIRECT_URI"),
            grant_type    : "authorization_code",
            client_secret : dotenv::var("MICROSOFT_AZURE_CLIENT_SECRET")
        })?);
    let mut response = request.send().await.map_err(|err| {
        surf::Error::from_str(err.status(), format!("Failed to exchange Microsoft auth code for Microsoft access token: {}", err.into_inner()))
    })?;
    let status = response.status();
    if (! status.is_success()) {
        return Err(surf::Error::from_str(status, format!("Failed to exchange Microsoft auth code for Microsoft access token: {}",
            match (&response.body_json::<MicrosoftAccessTokenError>().await) {
                Ok(error) => &error.error,
                Err(_)    => status.canonical_reason()
            }
        )));
    }
    response.body_json::<MicrosoftAccessToken>().await
}

#[derive(Ser)]
struct MicrosoftTokenQuery<'l> {
    client_id     : &'static str,
    scope         : &'static str,
    code          : &'l str,
    redirect_uri  : &'static str,
    grant_type    : &'static str,
    client_secret : &'static str
}

#[derive(Deser)]
pub struct MicrosoftAccessTokenError {
    error : String
}

#[derive(Deser)]
pub struct MicrosoftAccessToken {
    pub access_token  : String,
    pub expires_in    : u64,
    pub refresh_token : String
}


pub async fn exchange_xbox_auth(client : &Client, microsoft_token : &str) -> surf::Result<XboxAuth> {
    let request = client.post("https://user.auth.xboxlive.com/user/authenticate")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(format!("{{\"Properties\":{{\"AuthMethod\":\"RPS\",\"SiteName\":\"user.auth.xboxlive.com\",\"RpsTicket\":\"d={microsoft_token}\"}},\"RelyingParty\":\"http://auth.xboxlive.com\",\"TokenType\":\"JWT\"}}"));
    let mut response = request.send().await.map_err(|err| {
        surf::Error::from_str(err.status(), format!("Failed to exchange Microsoft access token for XBOX auth token: {}", err.into_inner()))
    })?;
    let status = response.status();
    if (! status.is_success()) {
        return Err(surf::Error::from_str(status, format!("Failed to exchange Microsoft access token for XBOX auth token: {}", status.canonical_reason())));
    }
    let json = response.body_json::<XboxAuthDeser>().await?;
    Ok(XboxAuth {
        token    : json.token,
        userhash : json.display_claims.xui[0].uhs.clone()
    })
}

#[derive(Deser)]
struct XboxAuthDeser {
    #[serde(rename = "Token")]
    token : String,
    #[serde(rename = "DisplayClaims")]
    display_claims : XboxAuthDeserDisplayClaims
}
#[derive(Deser)]
struct XboxAuthDeserDisplayClaims {
    xui : [XboxAuthDeserDisplayClaim; 1]
}
#[derive(Deser)]
struct XboxAuthDeserDisplayClaim {
    uhs : String
}

#[derive(Debug)]
pub struct XboxAuth {
    pub token    : String,
    pub userhash : String
}


pub async fn exchange_xsts_token(client : &Client, xbox_token : &str) -> surf::Result<String> {
    let request = client.post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(format!("{{\"Properties\":{{\"SandboxId\":\"RETAIL\",\"UserTokens\":[\"{xbox_token}\"]}},\"RelyingParty\":\"rp://api.minecraftservices.com/\",\"TokenType\":\"JWT\"}}"));
    let mut response = request.send().await.map_err(|err| {
        surf::Error::from_str(err.status(), format!("Failed to exchange XBOX auth token for XSTS Minecraft token: {}", err.into_inner()))
    })?;
    let status = response.status();
    if (! status.is_success()) {
        return Err(surf::Error::from_str(status, match (response.body_json::<XstsTokenError>().await) {
            Ok(err) => format!("Failed to exchange XBOX auth token for XSTS Minecraft token: {}", XstsTokenErrorCode::from(err.code)),
            Err(_)  => format!("Failed to exchange XBOX auth token for XSTS Minecraft token: {}", status.canonical_reason())
        }));
    }
    Ok(response.body_json::<XstsTokenDeser>().await?.token)
}

#[derive(Deser)]
struct XstsTokenDeser {
    #[serde(rename = "Token")]
    token : String
}

#[derive(Deser)]
struct XstsTokenError {
    #[serde(rename = "XErr")]
    code : usize
}

enum XstsTokenErrorCode {
    Banned,
    NoXbox,
    UnavailableCountry,
    AdultVerifRequired,
    Underage,
    Unknown
}
impl From<usize> for XstsTokenErrorCode {
    fn from(value : usize) -> Self {
        match (value) {
            2148916227   => Self::Banned,
            2148916233   => Self::NoXbox,
            2148916235   => Self::UnavailableCountry,
            2148916236
            | 2148916237 => Self::AdultVerifRequired,
            2148916238   => Self::Underage,
            2148916262
            | _          => Self::Unknown
        }
    }
}
impl fmt::Display for XstsTokenErrorCode {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self) {
            XstsTokenErrorCode::Banned             => write!(f, "Account is banned from Xbox"),
            XstsTokenErrorCode::NoXbox             => write!(f, "Account does not have an Xbox profile"),
            XstsTokenErrorCode::UnavailableCountry => write!(f, "Xbox is unavailable in account country"),
            XstsTokenErrorCode::AdultVerifRequired => write!(f, "Account needs adult verification on Xbox (South Korea)"),
            XstsTokenErrorCode::Underage           => write!(f, "Account is underage and needs to be added to a Family."),
            XstsTokenErrorCode::Unknown            => write!(f, "Unknown error")
        }
    }
}


pub async fn exchange_minecraft_token(client : &Client, user_hash : &str, xsts_token : &str) -> surf::Result<String> {
    let request = client.post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(format!("{{\"identityToken\":\"XBL3.0 x={user_hash};{xsts_token}\"}}"));
    let mut response = request.send().await.map_err(|err| {
        surf::Error::from_str(err.status(), format!("Failed to exchange XSTS Minecraft token for Minecraft access token: {}", err.into_inner()))
    })?;
    let status = response.status();
    if (! status.is_success()) {
        return Err(surf::Error::from_str(status, format!("Failed to exchange XSTS Minecraft token for Minecraft access token: {}",
            match (&response.body_json::<MinecraftTokenError>().await) {
                Ok(err) => &err.error,
                Err(_)  => status.canonical_reason()
            }
        )));
    }
    Ok(response.body_json::<MinecraftTokenDeser>().await?.access_token)
}

#[derive(Deser)]
struct MinecraftTokenDeser {
    access_token : String
}

#[derive(Deser)]
struct MinecraftTokenError {
    error : String
}
