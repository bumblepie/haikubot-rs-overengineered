use regex::Regex;

pub fn is_valid_haiku_id(id: &str) -> bool {
    lazy_static! {
        static ref HAIKU_ID_REGEX: Regex = Regex::new(r"^0x\d+$").unwrap();
    }
    HAIKU_ID_REGEX.is_match(&id)
}
