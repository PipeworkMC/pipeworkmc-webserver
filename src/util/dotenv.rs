use std::{
    cell::LazyCell,
    collections::HashMap,
    fs::File, io::Read
};


static mut DOTENV : LazyCell<HashMap<String, String>> = LazyCell::new(|| HashMap::new());


pub unsafe fn load() {
    #[allow(static_mut_refs)]
    unsafe { DOTENV.clear(); }
    if let Ok(mut f) = File::open(".env") {
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();
        for line in buf.lines() {
            if (line.starts_with('#')) { continue; }
            let mut line  = line.split('=');
            let     key   = line.next().unwrap().trim();
            if (key.is_empty()) { continue; }
            let     value = line.flat_map(|s| ["=", s]).skip(1).collect::<String>();
            #[allow(static_mut_refs)]
            unsafe { DOTENV.insert(key.to_string(), value); }
        }
    }
}


// pub fn try_var(key : &str) -> Option<&'static str> {
//     #[allow(static_mut_refs)]
//     unsafe{ DOTENV.get(key).map(|s| s.as_str()) }
// }

pub fn var(key : &str) -> &'static str {
    #[allow(static_mut_refs)]
    unsafe{ DOTENV.get(key).expect("missing dotenv key").as_str() }
}
