// acl_data.rs
use serde_derive::{Deserialize, Serialize};
use crate::prelude::{String, Vec};

#[cfg(feature = "std")]
use core::convert::TryFrom;
#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::BufReader;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclData {
  pub roles: Option<Vec<(String, Option<Vec<String>>)>>,
  pub resources: Option<Vec<(String, Option<Vec<String>>)>>,
  pub allow: Option<Vec<(String, Option<Vec<(String, Option<Vec<String>>)>>)>>,
  pub deny: Option<Vec<(String, Option<Vec<(String, Option<Vec<String>>)>>)>>,
}

#[cfg(feature = "std")]
impl<'a> TryFrom<&'a mut File> for AclData {
    type Error = serde_json::Error;

    fn try_from(file: &mut File) -> Result<Self, Self::Error> {
        let buf = BufReader::new(file);
        serde_json::from_reader(buf)
    }
}
