// acl_data.rs
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclData {
  pub roles: Option<Vec<(String, Option<Vec<String>>)>>,
  pub resources: Option<Vec<(String, Option<Vec<String>>)>>,
  pub allow: Option<Vec<(String, Option<Vec<(String, Option<Vec<String>>)>>)>>,
  pub deny: Option<Vec<(String, Option<Vec<(String, Option<Vec<String>>)>>)>>,
}

impl<'a> From<&'a mut File> for AclData {
    fn from(file: &mut File) -> Self {
        // let mut contents = String::new();
        // file.read_to_string(&mut contents);
        let buf = BufReader::new(file);
        serde_json::from_reader(buf).unwrap()
    }
}
