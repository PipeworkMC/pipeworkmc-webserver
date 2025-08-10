#![feature(decl_macro)]


use tide::{
    Endpoint,
    Request,
    Response
};


mod auth;

mod layout;
mod site;
use site::SharedSiteState;

mod util;
use util::dotenv;


fn main() -> tide::Result<()> { smol::block_on(async {
    unsafe { dotenv::load(); }
    let mut app = tide::with_state(SharedSiteState::default());

    app.at("/dashboard/login").get(handled(site::dashboard::login::route_login));
    app.at("/dashboard/oauth/microsoft").get(handled(site::dashboard::login::route_oauth_microsoft));
    app.at("/dashboard").get(handled(site::dashboard::route_index));

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}) }


fn handled<State, F, Fut, R>(f : F)
    -> impl Endpoint<State>
where
    State
        : Clone
        + Send + Sync
        + 'static,
    F
        : (Fn(Request<State>) -> Fut)
        + Clone + Copy
        + Send + Sync
        + 'static,
    Fut
        : Future<Output = tide::Result<R>>
        + Send,
    R
        : Into<Response>
{ move |req| async move {

    let mut res = match (f(req).await) {
        Ok(res)  => res.into(),
        Err(err) => Response::from(err)
    };

    let status = res.status();
    let err    = match (res.error()) {
        Some(err) => Some(layout::html!{ (err) }),
        None      => {
            if (status.is_client_error() || status.is_server_error()) {
                Some(layout::html!{ (status.canonical_reason()) })
            } else { None }
        }
    };

    if let Some(err_body) = err {
        res = Response::from(layout::default(
            status.canonical_reason(),
            layout::html!{ (status as usize) },
            true,
            layout::html!{
                div .content_centre {
                    p { strong { (err_body) } }
                    br;
                    div .icon_rows {
                        a href="/" {
                            (layout::icon_svg!("home.svg"))
                            span { "Home" }
                        }
                        a href="/dashboard" {
                            (layout::icon_svg!("dashboard.svg"))
                            span { "Dashboard" }
                        }
                    }
                }
            }
        ));
        res.set_status(status);
    }

    Ok(res)
} }
