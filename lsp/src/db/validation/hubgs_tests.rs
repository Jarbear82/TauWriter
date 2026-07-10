use crate::RootDatabase;
use crate::db::{self, HubValue};
use super::hubgs::*;
use super::Multiplicity;

#[test]
fn test_multiplicity_parse_and_validate() {
    // Test Exact
    let m1 = Multiplicity::parse("(1)");
    assert!(m1.validate(1));
    assert!(!m1.validate(0));
    assert!(!m1.validate(2));

    // Test Range *
    let m2 = Multiplicity::parse("*");
    assert!(m2.validate(1));
    assert!(m2.validate(100));
    assert!(!m2.validate(0));

    // Test Range min..max
    let m3 = Multiplicity::parse("1..3");
    assert!(m3.validate(1));
    assert!(m3.validate(2));
    assert!(m3.validate(3));
    assert!(!m3.validate(0));
    assert!(!m3.validate(4));

    // Test Range min..*
    let m4 = Multiplicity::parse("2..*");
    assert!(m4.validate(2));
    assert!(m4.validate(5));
    assert!(!m4.validate(1));
}

#[test]
fn test_validate_value_type_primitives() {
    let db = RootDatabase::default();
    let workspace = db::Workspace::new(&db, vec![]);

    // Test Text
    let val_text = HubValue::Text("hello".to_string());
    assert!(validate_value_type(&db, workspace, &val_text, "Text"));
    assert!(!validate_value_type(&db, workspace, &val_text, "Number"));

    // Test Number
    let val_num = HubValue::Number(db::RawF64::from_f64(12.3));
    assert!(validate_value_type(&db, workspace, &val_num, "Number"));
    assert!(!validate_value_type(&db, workspace, &val_num, "Boolean"));

    // Test Boolean
    let val_bool = HubValue::Boolean(true);
    assert!(validate_value_type(&db, workspace, &val_bool, "Boolean"));
    assert!(!validate_value_type(&db, workspace, &val_bool, "Text"));

    // Test Image
    let val_img_valid = HubValue::Text("avatar.png".to_string());
    let val_img_invalid = HubValue::Text("avatar.txt".to_string());
    assert!(validate_value_type(&db, workspace, &val_img_valid, "Image"));
    assert!(!validate_value_type(&db, workspace, &val_img_invalid, "Image"));
}
