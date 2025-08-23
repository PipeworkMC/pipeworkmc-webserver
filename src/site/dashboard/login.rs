use crate::{
    auth, layout,
    site::{ self, SharedSiteState }
};
use std::sync::Arc;
use tide::{
    Request,
    Response
};
use surf::Client;
use serde::Deserialize as Deser;


pub async fn route_login(req : &mut Request<SharedSiteState>) -> tide::Result<Response> {
    let login = Arc::clone(req.state()).lookup_login_session(req).await;
    site::require_logged_out!(login);

    Ok(tide::Response::from(layout::default(req,
        layout::PageType::Normal,
        login.as_ref().map(|l| &**l),
        "Dashboard",
        "Log In",
        layout::html!{
            div .content_centre {
                a href=(req.state().microsoft_oauth_url) {
                    (layout::icon_svg!("brand/microsoft_signin_dark.svg"))
                }
            }
        }
    ).await))
}


#[derive(Deser)]
struct MicrosoftOauthQuery {
    #[serde(rename = "code")]
    microsoft_code : String
}

pub async fn route_after_oauth(req : &mut Request<SharedSiteState>) -> tide::Result<Response> {
    {
        let login = Arc::clone(req.state()).lookup_login_session(req).await;
        site::require_logged_out!(login);
    }

    let query = req.query::<MicrosoftOauthQuery>()?;

    let client = Client::new();
    let microsoft_token   = auth::minecraft::login::exchange_microsoft_token(&client, &query.microsoft_code).await?;
    let xbox_auth         = auth::minecraft::login::exchange_xbox_auth(&client, &microsoft_token.access_token).await?;
    let xsts_token        = auth::minecraft::login::exchange_xsts_token(&client, &xbox_auth.token).await?;
    let minecraft_token   = auth::minecraft::login::exchange_minecraft_token(&client, &xbox_auth.userhash, &xsts_token).await?;
                            // auth::minecraft::account::verify_account_product(&client, &minecraft_token).await?;
    let minecraft_profile = auth::minecraft::account::fetch_account_profile(&client, &minecraft_token).await?;
    let minecraft_skin    = minecraft_profile.get_active_skin(&client).await?;

    Arc::clone(req.state()).create_login_session(req,
        minecraft_profile.uuid,
        minecraft_profile.username,
        minecraft_skin
    ).await;

    Ok(layout::html!{
        body {
            script { (layout::PreEscaped("window.location.replace(\"/dashboard\");")) }
        }
    }.into())
}
