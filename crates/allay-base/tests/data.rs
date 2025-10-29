use allay_base::data::*;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_basic_types() {
    let string_data = AllayData::from("hello");
    assert!(string_data.is_str());

    let int_data = AllayData::from(42);
    assert!(int_data.is_int());

    let bool_data = AllayData::from(true);
    assert!(bool_data.is_bool());
}

#[test]
fn test_list_operations() {
    let list = AllayData::from(AllayList::from([
        Arc::new(AllayData::from(1)),
        Arc::new(AllayData::from("test")),
    ]));
    let list = list.as_list().unwrap();

    assert_eq!(list.len(), 2);
}

#[test]
fn test_object_operations() {
    let mut obj = HashMap::new();
    obj.insert("key1".to_string(), Arc::new(AllayData::from("value1")));
    obj.insert("key2".to_string(), Arc::new(AllayData::from(42)));

    let data = AllayData::from(obj).as_obj().unwrap();
    assert!(data.contains_key("key1"));
}

#[test]
fn test_from_traits() {
    let from_str: AllayData = "hello".into();
    assert!(from_str.is_str());

    let from_i32: AllayData = 42.into();
    assert!(from_i32.is_int());

    let from_bool: AllayData = true.into();
    assert!(from_bool.is_bool());
}

#[test]
fn test_yaml_parsing() -> DataResult<()> {
    let yaml_str = r#"
key1: value1
key2: 42
key3:
    - item1
    - item2
    - item3
key4:
    subkey1: subvalue1
    subkey2: 100
"#;
    let data = AllayData::from_yaml(yaml_str)?;
    assert_eq!(data.get("key1").unwrap().as_str()?, "value1");
    assert_eq!(data.get("key2").unwrap().as_int()?, 42);
    assert_eq!(data.get("key3").unwrap().as_list()?.len(), 3);
    assert_eq!(
        data.get("key4").unwrap().as_obj()?.get("subkey1").unwrap().as_str()?,
        "subvalue1"
    );

    Ok(())
}

#[test]
fn test_toml_parsing() -> DataResult<()> {
    let toml_str = r#"
key1 = "value1"
key2 = [ "item1", "item2", "item3" ]
[key3]
subkey1 = "subvalue1"
subkey2 = 100
"#;
    let data = AllayData::from_toml(toml_str)?;
    assert_eq!(data.get("key1").unwrap().as_str()?, "value1");
    assert_eq!(data.get("key2").unwrap().as_list()?.len(), 3);
    assert_eq!(
        data.get("key3").unwrap().as_obj()?.get("subkey1").unwrap().as_str()?,
        "subvalue1"
    );
    Ok(())
}
