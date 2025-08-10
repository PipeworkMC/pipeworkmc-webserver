use crate::util;
use std::borrow::Cow;
use tide::StatusCode;
use surf::Client;
use serde::Deserialize as Deser;
use uuid::Uuid;
use image::{ ImageBuffer, Rgba };


const STEVE_SKIN : &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAgAAAAICAYAAADED76LAAAAAXNSR0IB2cksfwAAAARnQU1BAACxjwv8YQUAAAAgY0hSTQAAeiYAAICEAAD6AAAAgOgAAHUwAADqYAAAOpgAABdwnLpRPAAAAAlwSFlzAAAuIwAALiMBeKU/dgAAAAd0SU1FB+kICQEQGZ+MNAcAAADPSURBVBjTTcyrbgJBAIXhn83MLC0ssxWQPkANwRAwdcXyADxBTV8GX4LAkSZVBIElBENIKmoQyGaraLLDZe/J1i0ceU7OV+o+PeQAVSW4zTnJiJIMy5zDorgdq0pQVgJhK8nbyzOPjTri3iELToRxijE+o9WG0vi1n2vtcmdLAMI4LSRjfCytXQbDKZN1q3h+bNsMhlO0dq8CwPJrB0Cv07wKlyDAO3hM5gscS+JYkvfPGd7B49f3EcLO+TulxFHC98+eOEqwy4rjJaVWkfwD+DdX0n69wvwAAAAASUVORK5CYII=";


pub async fn verify_account_product(
    client          : &Client,
    minecraft_token : &str
) -> surf::Result<()> {
    let request = client.get("https://api.minecraftservices.com/entitlements/mcstore")
        .header("Authorization", format!("Bearer {minecraft_token}"));
    let mut response = request.send().await.map_err(|err| {
        surf::Error::from_str(err.status(), format!("Failed to verify Minecraft product license: {}", err.into_inner()))
    })?;
    let mut status = response.status();
    if (status.is_success()) {
        if let Ok(entitlements) = response.body_json::<MinecraftAccountEntitlements>().await {

            return if (entitlements.items.iter().any(|e| e.name.as_ref().is_some_and(|n| n == "product_minecraft"))
                && entitlements.items.iter().any(|e| e.name.as_ref().is_some_and(|n| n == "game_minecraft"))
            ) { Ok(()) }
            else {
                Err(surf::Error::from_str(StatusCode::Unauthorized, format!("Failed to verify Minecraft product license: Account does not own Minecraft Java Edition")))
            }

        } else {
            status = StatusCode::InternalServerError;
        }
    }
    return Err(surf::Error::from_str(status, format!("Failed to verify Minecraft product license: {}", status.canonical_reason())));
}

#[derive(Deser, Debug)]
pub struct MinecraftAccountEntitlements {
    items : Vec<MinecraftAccountEntitlement>
}
#[derive(Deser, Debug)]
pub struct MinecraftAccountEntitlement {
    name : Option<String>
}


pub async fn fetch_account_profile(
    client          : &Client,
    minecraft_token : &str
) -> surf::Result<MinecraftAccountProfile> {
    let request = client.get("https://api.minecraftservices.com/minecraft/profile")
        .header("Authorization", format!("Bearer {minecraft_token}"));
    let mut response = request.send().await.map_err(|err| {
        surf::Error::from_str(err.status(), format!("Failed to fetch Minecraft account profile: {}", err.into_inner()))
    })?;
    let status = response.status();
    if (! status.is_success()) {
        return Err(surf::Error::from_str(status, format!("Failed to fetch Minecraft account profile: {}", status.canonical_reason())));
    }
    response.body_json::<MinecraftAccountProfile>().await
}

#[derive(Deser, Debug)]
pub struct MinecraftAccountProfile {
    #[serde(rename = "id")]
    pub uuid     : Uuid,
    #[serde(rename = "name")]
    pub username : String,
    pub skins    : Vec<MinecraftAccountSkin>
}

#[derive(Deser, Debug)]
pub struct MinecraftAccountSkin {
    pub state   : MinecraftAccountSkinState,
    pub url     : String
}

#[derive(Deser, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MinecraftAccountSkinState {
    #[serde(rename = "ACTIVE")]
    Active,
    #[serde(rename = "INACTIVE")]
    Inactive
}

impl MinecraftAccountProfile {

    pub async fn get_active_skin(&self, client : &Client) -> surf::Result<Cow<'static, str>> {
        let active_skin = self.skins.iter().find_map(|skin| (skin.state == MinecraftAccountSkinState::Active).then(|| &skin.url));
        Ok(match (active_skin) {
            Some(skin_url) => {
                let     image_full = util::image::fetch(client, skin_url).await?.to_rgba32f();
                let mut container  = [0u8; 8*8*4];
                let mut image_face = ImageBuffer::<Rgba<u8>, _>::from_raw(8, 8, container.as_mut_slice()).unwrap();
                for (x, y, px,) in image_face.enumerate_pixels_mut() {
                    let head = image_full.get_pixel(8 + x, 8 + y);
                    let cap  = image_full.get_pixel(40 + x, 8 + y);
                    px.0[0] = (util::math::lerp(head.0[0]*head.0[3], cap.0[0], cap.0[3]).clamp(0.0, 1.0) * (u8::MAX as f32)) as u8;
                    px.0[1] = (util::math::lerp(head.0[1]*head.0[3], cap.0[1], cap.0[3]).clamp(0.0, 1.0) * (u8::MAX as f32)) as u8;
                    px.0[2] = (util::math::lerp(head.0[2]*head.0[3], cap.0[2], cap.0[3]).clamp(0.0, 1.0) * (u8::MAX as f32)) as u8;
                    px.0[3] = u8::MAX;
                }
                Cow::Owned(util::image::to_base64(&image_face)?)
            },
            None => Cow::Borrowed(STEVE_SKIN)
        })
    }

}
