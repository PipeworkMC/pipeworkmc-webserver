use maud::{
    DOCTYPE,
    PreEscaped,
    Render
};
use chrono::{ Datelike, Utc };


pub fn default(
    supertitle : impl Render,
    title      : impl Render,
    is_error   : bool,
    main       : impl Render
) -> PreEscaped<String> {
    html!{

        head {
            (stylesheet!("main.css"))
        }

        body {

            div #header {
                div #header_page .header_error[is_error] {
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
                    span .no_account { "No" (NBSP) "Account" }
                    (icon_svg!("account.svg"))
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
