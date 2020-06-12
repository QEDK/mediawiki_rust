/*!
The `User` class deals with the (current) Api user.
*/

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use crate::api::Api;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

/// `User` contains the login data for the `Api`
#[derive(Debug, Default, Clone)]
pub struct User {
    lgusername: Option<String>,
    lguserid: Option<u64>,
    is_logged_in: bool,
    user_info: Option<Value>,
}

impl User {
    /// Returns a new, blank, not-logged-in user
    pub fn new() -> User {
        User {
            lgusername: None,
            lguserid: None,
            is_logged_in: false,
            user_info: None,
        }
    }

    /// Checks if the user is logged in
    pub fn logged_in(&self) -> bool {
        self.is_logged_in
    }

    /// Checks is the user has a spefic right (e.g. "bot", "autocinfirmed")
    pub fn has_right(&self, right: &str) -> Option<bool> {
        match &self.user_info {
            Some(ui) => {
                ui["query"]["userinfo"]["rights"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter(|x| x.as_str().unwrap_or("") == right)
                    .count()
                    > 0
            }
            None => None,
        }
    }

    /// Checks if the user has a bot flag
    pub fn is_bot(&self) -> Option<bool> {
        self.has_right("bot")
    }

    /// Checks if the user is autoconfirmed
    pub fn is_autoconfirmed(&self) -> Option<bool> {
        self.has_right("autoconfirmed")
    }

    /// Checks if the user is allowed to edit
    pub fn can_edit(&self) -> Option<bool> {
        self.has_right("edit")
    }

    /// Checks if the user is allowed to create a page
    pub fn can_create_page(&self) -> Option<bool> {
        self.has_right("createpage")
    }

    /// Checks if the user is allowed to upload a file
    pub fn can_upload(&self) -> Option<bool> {
        self.has_right("upload")
    }

    /// Checks if the user is allowed to move (rename) a page
    pub fn can_move(&self) -> Option<bool> {
        self.has_right("move")
    }

    /// Checks if the user is allowed to patrol edits
    pub fn can_patrol(&self) -> Option<bool> {
        self.has_right("patrol")
    }

    /// Loads the user info, which is stored in the object; returns Ok(()) if successful
    pub fn load_user_info(&mut self, api: &Api) -> Result<(), Box<dyn Error>> {
        match self.user_info {
            Some(_) => Ok(()),
            None => {
                let params: HashMap<String, String> = vec![
                    ("action", "query"),
                    ("meta", "userinfo"),
                    ("uiprop", "blockinfo|groups|groupmemberships|implicitgroups|rights|options|ratelimits|realname|registrationdate|unreadcount|centralids|hasmsg"),
                ]
                .iter()
                .map(|x| (x.0.to_string(), x.1.to_string()))
                .collect();
                let res = api.query_api_json(&params, "GET")?;
                self.user_info = Some(res);
                Ok(())
            }
        }
    }

    /// Returns Ok(user name) (None if not logged in)
    pub fn user_name(&self) -> Option<String> {
        self.lgusername
    }

    /// Returns Ok(user id) (None if not logged in)
    pub fn user_id(&self) -> Option<u64> {
        self.lguserid
    }

    /// Tries to set user information from the `Api` call
    pub fn set_from_login(&mut self, login: &Value) -> Result<(), &str> {
        if login["result"] == "Success" {
            match login["lgusername"].as_str() {
                Some(s) => self.lgusername = Some(s.to_string()),
                None => return Err("No lgusername in login result"),
            }
            match login["lguserid"].as_u64() {
                Some(u) => self.lguserid = Some(u),
                None => return Err("No lguserid in login result"),
            }

            self.is_logged_in = true;
        } else {
            self.is_logged_in = false;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::*;

    fn wd_api() -> &'static Api {
        lazy_static! {
            static ref API: Api = Api::new("https://www.wikidata.org/w/api.php").unwrap();
        }
        &API
    }

    #[test]
    fn user_not_logged_in_by_default() {
        let user = User::new();
        assert!(!user.logged_in());
    }

    #[test]
    fn user_login() {
        let user_name = "test user 1234";
        let user_id = 12345;
        let mut user = User::new();
        let login = json!({"result":"Success","lgusername":user_name,"lguserid":user_id});
        user.set_from_login(&login).unwrap();
        assert!(user.logged_in());
        assert_eq!(user.user_name(), user_name);
        assert_eq!(user.user_id(), user_id);
    }

    #[test]
    fn user_rights() {
        let mut user = User::new();
        user.load_user_info(wd_api()).unwrap();
        assert!(!user.is_bot());
        assert!(user.can_edit());
        assert!(!user.can_upload());
        assert!(user.has_right("createaccount"));
        assert!(!user.has_right("thisisnotaright"));
    }
}
