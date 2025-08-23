use rand::CryptoRng;


const CHARS : &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-.";


fn gen_bytes_with<const LEN : usize>(rand : &mut impl CryptoRng) -> [u8; LEN] {
    let mut dst = [0u8; LEN];
    rand.fill_bytes(&mut dst);
    dst
}

fn gen_token_with<const LEN : usize>(rand : &mut impl CryptoRng) -> String {
    let bytes = gen_bytes_with::<LEN>(rand);
    unsafe { String::from_utf8_unchecked(bytes.into_iter().map(|b| {
        CHARS[(b as usize).rem_euclid(CHARS.len())]
    }).collect::<Vec<_>>()) }
}


#[inline]
pub fn gen_token() -> String {
    gen_token_with::<256>(&mut rand::rng())
}
