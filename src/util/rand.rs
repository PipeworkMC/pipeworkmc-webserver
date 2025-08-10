use rand::CryptoRng;


const CHARS : &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-.";


fn gen_urlsafe_randstring<const LEN : usize>(rand : &mut impl CryptoRng) -> String {
    let mut dst = [0u8; LEN];
    rand.fill_bytes(&mut dst);
    unsafe { String::from_utf8_unchecked(dst.into_iter().map(|b| {
        CHARS[(b as usize).rem_euclid(CHARS.len())]
    }).collect::<Vec<_>>()) }
}

#[inline]
pub fn gen_oauth_state_string() -> String {
    gen_urlsafe_randstring::<16>(&mut rand::rng())
}
