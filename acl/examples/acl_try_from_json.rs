use std::fs::File;
use walrs_acl::simple::AclBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "./acl/test-fixtures/example-acl-allow-and-deny-rules.json";
    let mut f = File::open(&file_path)?;
    let acl = AclBuilder::try_from(&mut f)?.build()?;

    println!("ACL: {:#?}", acl);
    // ...
    Ok(())
}
