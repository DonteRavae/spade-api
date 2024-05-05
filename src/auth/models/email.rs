use fancy_regex::Regex;

#[derive(Debug)]
pub struct Email(String);

impl Email {
    pub fn parse(email: String) -> Result<Self, String> {
        let email_validation_test = Regex::new(r"^[\w\-\.]+@([\w-]+\.)+[\w-]{2,}$").unwrap();
        match email_validation_test.is_match(&email) {
            Ok(result) => {
                if result {
                    Ok(Email(email.to_ascii_lowercase()))
                } else {
                    Err("Please enter a valid email or password".to_string())
                }
            }
            Err(_) => {
                Err("We seem to be having an error on our end. Please try again.".to_string())
            }
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
