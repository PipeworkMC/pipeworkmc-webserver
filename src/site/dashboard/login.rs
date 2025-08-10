use crate::{
    auth, layout,
    site::SharedSiteState
};
use tide::{
    Redirect,
    Request,
    Response
};
use surf::Client;
use serde::Deserialize as Deser;


pub async fn route_login(req : Request<SharedSiteState>) -> tide::Result<Response> {
    Ok(tide::Response::from(layout::default(
        "Dashboard",
        "Log In",
        false,
        layout::html!{
            div .content_centre {
                a href=(req.state().microsoft_oauth_url) {
                    (layout::icon_svg!("brand/microsoft_signin_dark.svg"))
                }
            }
        }
    )))
}


#[derive(Deser)]
struct MicrosoftOauthQuery {
    #[serde(rename = "code")]
    microsoft_code : String
}

pub async fn route_oauth_microsoft(req : Request<SharedSiteState>) -> tide::Result<Redirect<&'static str>> {
    let query = req.query::<MicrosoftOauthQuery>()?;

    let client = Client::new();
    let microsoft_token   = auth::minecraft::login::exchange_microsoft_token(&client, &query.microsoft_code).await?;
    let xbox_auth         = auth::minecraft::login::exchange_xbox_auth(&client, &microsoft_token.access_token).await?;
    let xsts_token        = auth::minecraft::login::exchange_xsts_token(&client, &xbox_auth.token).await?;
    let minecraft_token   = auth::minecraft::login::exchange_minecraft_token(&client, &xbox_auth.userhash, &xsts_token).await?;
                            auth::minecraft::account::verify_account_product(&client, &minecraft_token).await?;
    let minecraft_profile = auth::minecraft::account::fetch_account_profile(&client, &minecraft_token).await?;
    let skin_uri          = minecraft_profile.get_active_skin(&client).await?;

    // TODO: Login

    Ok(Redirect::see_other("/dashboard"))
}
