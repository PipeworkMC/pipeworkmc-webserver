use crate::{
    auth,
    util::rand
};
use pipeworkmc_db::{ PipeworkDb, LoginSession };
use std::{
    collections::HashMap,
    sync::Arc
};
use tide::{
    Request,
    Response,
    StatusCode
};
use smol::lock::RwLock;
use uuid::Uuid;


pub mod dashboard;


pub type SharedSiteState = Arc<SiteState>;

pub struct SiteState {
    microsoft_oauth_url : String,
    db                  : PipeworkDb,
    login_sessions      : RwLock<HashMap<Uuid, Arc<LoginSession>>>
}

impl SiteState {

    pub fn new(db : PipeworkDb) -> SharedSiteState {
        Arc::new(SiteState {
            microsoft_oauth_url : auth::minecraft::login::build_microsoft_access_code_url(None),
            db,
            login_sessions      : RwLock::new(HashMap::new())
        })
    }

    pub async fn lookup_login_session(&self, req : &mut Request<SharedSiteState>) -> Option<Arc<LoginSession>> {
        let session        = req.session_mut();
        let minecraft_uuid = Uuid::parse_str(&session.get_raw("minecraft-uuid")?).ok()?;
        let sessionkey     = session.get_raw("pipeworkmc-sessionkey")?;
        if let Some(entry) = self.login_sessions.read().await.get(&minecraft_uuid) {
            if (sessionkey == entry.sessionkey) {
                return Some(Arc::clone(entry));
            }
        }
        if let Some(entry) = self.db.lookup_login_session(minecraft_uuid).await.ok().flatten() {
            if (sessionkey == entry.sessionkey) {
                let login = Arc::new(entry);
                self.login_sessions.write().await.insert(minecraft_uuid, Arc::clone(&login));
                return Some(login);
            }
        }
        session.remove("minecraft-uuid");
        session.remove("pipeworkmc-sessionkey");
        None
    }

    pub async fn create_login_session(&self,
        req                : &mut Request<SharedSiteState>,
        minecraft_uuid     : Uuid,
        minecraft_username : String,
        minecraft_skin     : Option<String>
    ) {
        let sessionkey = rand::gen_token();
        {
            let session = req.session_mut();
            session.insert_raw("pipeworkmc-sessionkey", sessionkey.clone());
            session.insert_raw("minecraft-uuid", minecraft_uuid.to_string());
        }
        let login = Arc::new(LoginSession {
            sessionkey,
            minecraft_username,
            minecraft_skin
        });
        self.login_sessions.write().await.insert(minecraft_uuid, Arc::clone(&login));
        _ = self.db.create_login_session(minecraft_uuid, &login).await;
    }

}


pub async fn route_todo(_ : &mut Request<SharedSiteState>) -> tide::Result<Response> {
    Err(tide::Error::from_str(StatusCode::InternalServerError, "Page under construction"))
}


pub macro require_logged_out($login:expr) {
    if (($login).is_some()) {
        return Ok(tide::Redirect::see_other("/dashboard").into());
    }
}

pub macro require_logged_in($login:expr) {
    if (($login).is_none()) {
        return Ok(tide::Redirect::see_other("/dashboard/login").into());
    }
}
