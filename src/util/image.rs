use core::ops::Deref;
use std::io::Cursor;
use surf::Client;
use image::{
    DynamicImage,
    EncodableLayout,
    ImageBuffer,
    ImageFormat,
    ImageReader,
    ImageResult,
    Pixel,
    PixelWithColorType
};
use base64::{
    prelude::BASE64_STANDARD,
    Engine
};


pub async fn fetch(
    client    : &Client,
    image_url : &str
) -> surf::Result<DynamicImage> {
    let     request  = client.get(image_url);
    let mut response = request.send().await?;
    let     bytes    = response.body_bytes().await?;
    let reader   = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?;
    Ok(reader.decode()?)
}


pub fn to_base64<P, Container>(image : &ImageBuffer<P, Container>) -> ImageResult<String>
where
    P             : Pixel + PixelWithColorType,
    [P::Subpixel] : EncodableLayout,
    Container     : Deref<Target = [P::Subpixel]>
{
    let mut bytes = Vec::new();
    image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
    let mut string = String::new();
    BASE64_STANDARD.encode_string(&bytes, &mut string);
    Ok(format!("data:image/png;base64,{string}"))
}
