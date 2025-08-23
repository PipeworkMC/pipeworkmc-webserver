use crate::site::SharedSiteState;
use pipeworkmc_db::LoginSession;
use std::sync::Arc;
use tide::Request;
pub use maud::{
    DOCTYPE,
    PreEscaped,
    Render
};
use chrono::{ Datelike, Utc };


const STEVE_SKIN : &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAgAAAAICAYAAADED76LAAAAAXNSR0IB2cksfwAAAARnQU1BAACxjwv8YQUAAAAgY0hSTQAAeiYAAICEAAD6AAAAgOgAAHUwAADqYAAAOpgAABdwnLpRPAAAAAlwSFlzAAAuIwAALiMBeKU/dgAAAAd0SU1FB+kICQEQGZ+MNAcAAADPSURBVBjTTcyrbgJBAIXhn83MLC0ssxWQPkANwRAwdcXyADxBTV8GX4LAkSZVBIElBENIKmoQyGaraLLDZe/J1i0ceU7OV+o+PeQAVSW4zTnJiJIMy5zDorgdq0pQVgJhK8nbyzOPjTri3iELToRxijE+o9WG0vi1n2vtcmdLAMI4LSRjfCytXQbDKZN1q3h+bNsMhlO0dq8CwPJrB0Cv07wKlyDAO3hM5gscS+JYkvfPGd7B49f3EcLO+TulxFHC98+eOEqwy4rjJaVWkfwD+DdX0n69wvwAAAAASUVORK5CYII=";


#[derive(PartialEq, Eq)]
pub enum PageType {
    Normal,
    Error
}


pub async fn default(
    req        : &mut Request<SharedSiteState>,
    page_type  : PageType,
    login      : Option<&LoginSession>,
    supertitle : impl Render,
    title      : impl Render,
    main       : impl Render
) -> PreEscaped<String> {
    let mut has_account  = false;
    let mut account_name = html!{ "No" (NBSP) "Account" };
    let mut account_icon = html!{ (icon_svg!("account.svg")) };
    if let Some(login) = login {
        has_account  = true;
        account_name = html!{ (login.minecraft_username) };
        account_icon = html!{ img src=(login.minecraft_skin.as_ref().map_or(STEVE_SKIN, |s| s.as_str())); };
    }

    html!{

        head {
            (stylesheet!("main.css"))
        }

        body {

            div #header {
                div #header_page .header_error[page_type == PageType::Error] {
                    span #header_page_supertitle { (supertitle) } (NBSP)
                    br;
                    span #header_page_title { (title) } (NBSP)
                }
                div #header_logo {
                    "PIPEW"
                    (icon_png_b64!("brand/pipework.png.b64"))
                     "RK"
                    span { "MC" }
                }
                div #header_account {
                    span .no_account[! has_account] { (account_name) }
                    (account_icon)
                }
            }

            div #main {
                (main)
            }

            div #footer {
                hr .above;
                div #footer_links {
                    a href="/" {
                        (icon_svg!("home.svg"))
                    }
                    (NBSP)
                    a .pad6 href="https://github.com/PipeworkMC" target="_blank" {
                        (icon_svg!("brand/github.svg"))
                    }
                    a .pad6 target="_blank" {
                        (icon_svg!("brand/matrix.svg"))
                    }
                    a href="mailto:pipeworkmc@duck.com" target="_blank" {
                        (icon_svg!("mail.svg"))
                    }
                }
                hr .below;
                div #footer_copyright {
                    span {
                        (COPY) " " (Utc::now().year()) " "
                    }
                    a href="https://github.com/Totobird-Creations" target="_blank" {
                        "Totobird-Creations"
                    }
                }
                div #footer_noassoc { "NOT AN OFFICIAL MINECRAFT SERVICE. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT." }
            }

        }

    }
}


pub const NBSP : PreEscaped<&str> = PreEscaped("&nbsp;");
pub const COPY : PreEscaped<&str> = PreEscaped("&copy;");

pub macro html($($tt:tt)*) {
    ::maud::html!{
        (DOCTYPE) html { $($tt)* }
    }
}

pub macro stylesheet($path:tt) {
    ::maud::html!{ style { (::maud::PreEscaped(::core::include_str!(::core::concat!(::core::env!("CRATE_ROOT"), "/assets/stylesheet/", $path)))) } }
}
pub macro icon_svg($path:tt) {
    ::maud::PreEscaped(::core::include_str!(::core::concat!(::core::env!("CRATE_ROOT"), "/assets/icon/", $path)))
}
pub macro icon_png_b64($path:tt) {
    ::maud::html!{ img src=(::core::concat!("data:image/png;base64,", ::core::include_str!(::core::concat!(::core::env!("CRATE_ROOT"), "/assets/icon/", $path)))); }
}
