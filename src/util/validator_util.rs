use fancy_regex::Regex;
use lazy_static::lazy_static;
use validator::ValidationError;

lazy_static! {
    static ref RE_PASSWORD: Regex = Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$").unwrap();
    static ref RE_USERNAME: Regex = Regex::new(r"[a-zA-Z][a-zA-Z0-9._]{4,32}$").unwrap();
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    match RE_PASSWORD.is_match(password) {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError::new("Password Error")),
    }
}

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    match RE_USERNAME.is_match(username) {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError::new("Username Error")),
    }
}