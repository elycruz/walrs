use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;

use walrs_acl::simple::AclData;

#[test]
pub fn test_from_file_ref() -> Result<(), Box<dyn std::error::Error>> {
  let file_path = "./test-fixtures/example-acl.json";

  // Get digraph data
  let mut f = File::open(&file_path)?;

  let _: AclData = AclData::try_from(&mut f)?;
  let _ = BufReader::new(f);

  // println!("{:?}", &acl_data);
  Ok(())
}
