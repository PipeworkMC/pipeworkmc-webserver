use crate::auth;
use std::{
    collections::HashMap,
    sync::Arc
};
use smol::lock::RwLock;


pub mod dashboard;


pub type SharedSiteState = Arc<SiteState>;

pub struct SiteState {
    microsoft_oauth_url : String,
    login_session       : RwLock<HashMap<String, LoginSession>>
}

impl Default for SiteState {
    fn default() -> Self {
        Self {
            microsoft_oauth_url : auth::minecraft::login::build_microsoft_access_code_url(None),
            login_session       : RwLock::new(HashMap::new())
        }
    }
}


pub struct LoginSession {}
