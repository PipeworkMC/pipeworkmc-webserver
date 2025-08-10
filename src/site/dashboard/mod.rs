use crate::site::SharedSiteState;
use tide::{ Request, Response, StatusCode };


pub mod login;


pub async fn route_index(req : Request<SharedSiteState>) -> tide::Result<Response> {
    Err(tide::Error::from_str(StatusCode::InternalServerError, "Page under construction"))
}
