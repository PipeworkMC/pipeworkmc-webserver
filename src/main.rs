#![feature(
    decl_macro,
    duration_constructors_lite,
    impl_trait_in_fn_trait_return
)]


use pipeworkmc_db::PipeworkDb;
use tide::{
    Request,
    Response,
    StatusCode,
    sessions::{
        SessionMiddleware,
        CookieStore
    }
};
use tide_rustls::TlsListener;


mod auth;

mod layout;
mod site;
use site::{ SiteState, SharedSiteState };

mod util;
use util::dotenv;


fn main() -> tide::Result<()> { smol::block_on(async {
    unsafe { dotenv::load(); }

    let mut db_addr      = dotenv::var("DATABASE_ADDRRESS").split(":");
    let     db_addr_host = db_addr.next().unwrap();
    let     db_addr_port = db_addr.flat_map(|s| ["s", s]).skip(1).collect::<String>();
    let     db_addr_port = if (db_addr_port.is_empty()) { 5432 } else { db_addr_port.parse::<u16>().unwrap() };
    let     db = PipeworkDb::connect(db_addr_host, db_addr_port).await.unwrap();

    let mut app = tide::with_state(SiteState::new(db));

    app.with(SessionMiddleware::new(
        CookieStore,
        dotenv::var("SESSION_SECRET").as_bytes()
    )
        .with_cookie_name("pipeworkmc")
    );

    app.at("/").get(handled!(site::route_todo));

    app.at("/dashboard/login").get(handled!(site::dashboard::login::route_login));
    app.at("/dashboard/login/after_oauth").get(handled!(site::dashboard::login::route_after_oauth));
    app.at("/dashboard").get(handled!(site::dashboard::route_index));

    app.at("*").get(handled!(async |_| tide::Result::<Response>::Err(tide::Error::from_str(
        StatusCode::NotFound,
        StatusCode::NotFound.canonical_reason()
    ))));

    app.listen(TlsListener::build()
        .addrs("127.0.0.1:8080")
        .cert("cert/pipeworkmc.cert")
        .key("cert/pipeworkmc.key")
    ).await?;
    Ok(())
}) }


macro handled($route:expr) {
    |mut req : Request<SharedSiteState>| async move {

        let mut res = match (($route)(&mut req).await) {
            Ok(res)  => Into::<Response>::into(res),
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

        let login = std::sync::Arc::clone(req.state()).lookup_login_session(&mut req).await;
        if let Some(err_body) = err {
            res = Response::from(layout::default(&mut req,
                crate::layout::PageType::Error,
                login.as_ref().map(|l| &**l),
                status.canonical_reason(),
                status as usize,
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
            ).await);
            res.set_status(status);
        }

        Ok(res)

    }
}
