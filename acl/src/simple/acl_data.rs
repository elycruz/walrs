// acl_data.rs
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclData {
  pub roles: Option<Vec<(String, Option<Vec<String>>)>>,
  pub resources: Option<Vec<(String, Option<Vec<String>>)>>,
  pub allow: Option<Vec<(String, Option<Vec<(String, Option<Vec<String>>)>>)>>,
  pub deny: Option<Vec<(String, Option<Vec<(String, Option<Vec<String>>)>>)>>,
}

impl<'a> TryFrom<&'a mut File> for AclData {
    type Error = serde_json::Error;

    fn try_from(file: &mut File) -> Result<Self, Self::Error> {
        let buf = BufReader::new(file);
        serde_json::from_reader(buf)
    }
}
