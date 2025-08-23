use crate::site::{ self, SharedSiteState };
use std::sync::Arc;
use tide::{ Request, Response, StatusCode };


pub mod login;


pub async fn route_index(req : &mut Request<SharedSiteState>) -> tide::Result<Response> {
    let login = Arc::clone(req.state()).lookup_login_session(req).await;
    site::require_logged_in!(login);

    Err(tide::Error::from_str(StatusCode::InternalServerError, "Page under construction"))
}
