use std::fs::File;
use std::io::BufReader;

use walrs_acl::simple::AclData;

#[test]
pub fn test_from_file_ref() -> Result<(), std::io::Error> {
  let file_path = "./test-fixtures/example-acl.json";

  // Get digraph data
  let mut f = File::open(&file_path)?;

  let acl_data: AclData = (&mut f).into();
  let b = BufReader::new(f);

  // println!("{:?}", &acl_data);
  Ok(())
}
