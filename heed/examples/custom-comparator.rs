use std::cmp::Ordering;
use std::error::Error;
use std::path::Path;
use std::{fs, str};
use std::borrow::Cow;

use heed::{DatabaseFlags, EnvOpenOptions};
use heed_traits::{Comparator, BytesDecode, BytesEncode, BoxedError};
use heed_types::{Str, Unit, U8};

enum FirstU64 {}

impl Comparator for FirstU64 {
    fn compare(a: &[u8], b: &[u8]) -> Ordering {
        let a: u64 = u64::from_le_bytes((&a[0..8]).try_into().unwrap());
        let b: u64 = u64::from_le_bytes((&b[0..8]).try_into().unwrap());
        a.cmp(&b)
    }
}

struct SubKeyValue {
    sub_key: u64,
    value: u128
}

impl<'a> BytesDecode<'a> for SubKeyValue {
    type DItem = Self;
    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, BoxedError> {
        Ok(Self {
            sub_key: u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            value: u128::from_le_bytes(bytes[8..].try_into().unwrap())
        })
    }
}

impl<'a> BytesEncode<'a> for SubKeyValue {
    type EItem = Self;

    fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, BoxedError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&item.sub_key.to_le_bytes());
        bytes.extend_from_slice(&item.value.to_le_bytes());
        Ok(bytes.into())
    }
}


fn main() -> Result<(), Box<dyn Error>>  {
    let env_path = Path::new("target").join("custom-key-cmp.mdb");

    let _ = fs::remove_dir_all(&env_path);

    fs::create_dir_all(&env_path)?;
    let env = EnvOpenOptions::new()
        .map_size(10 * 1024 * 1024) // 10MB
        .max_dbs(3)
        .open(env_path)?;

    let mut wtxn = env.write_txn()?;
    let db = env
        .database_options()
        .types::<U8, SubKeyValue>()
        .value_comparator::<FirstU64>()
        .flags(DatabaseFlags::DUP_SORT)
        .create(&mut wtxn)?;
    wtxn.commit()?;

    let mut wtxn = env.write_txn()?;

    // We fill our database with entries.
    db.put(&mut wtxn, &1, &SubKeyValue {
        sub_key: 10,
        value: 98456
    })?;
    db.put(&mut wtxn, &1, &SubKeyValue {
        sub_key: 5343,
        value: 64577654457
    })?;


    let (_, value) = db.get_key_value(&wtxn, &1, &SubKeyValue {
        sub_key: 10,
        value: 0 // set a default for now
    })?;

    assert_eq!(value.value, 98456);

    let (_, value) = db.get_key_value(&wtxn, &1, &SubKeyValue {
        sub_key: 5343,
        value: 0 // set a default for now
    })?;

    assert_eq!(value.value, 64577654457);

    Ok(())
}

/*
fn main() -> Result<(), Box<dyn Error>> {
    let env_path = Path::new("target").join("custom-key-cmp.mdb");

    let _ = fs::remove_dir_all(&env_path);

    fs::create_dir_all(&env_path)?;
    let env = EnvOpenOptions::new()
        .map_size(10 * 1024 * 1024) // 10MB
        .max_dbs(3)
        .open(env_path)?;

    let mut wtxn = env.write_txn()?;
    let db = env
        .database_options()
        .types::<Str, Unit>()
        .key_comparator::<StringAsIntCmp>()
        .create(&mut wtxn)?;
    wtxn.commit()?;

    let mut wtxn = env.write_txn()?;

    // We fill our database with entries.
    db.put(&mut wtxn, "-100000", &())?;
    db.put(&mut wtxn, "-10000", &())?;
    db.put(&mut wtxn, "-1000", &())?;
    db.put(&mut wtxn, "-100", &())?;
    db.put(&mut wtxn, "100", &())?;

    // We check that the key are in the right order ("-100" < "-1000" < "-10000"...)
    let mut iter = db.iter(&wtxn)?;
    assert_eq!(iter.next().transpose()?, Some(("-100000", ())));
    assert_eq!(iter.next().transpose()?, Some(("-10000", ())));
    assert_eq!(iter.next().transpose()?, Some(("-1000", ())));
    assert_eq!(iter.next().transpose()?, Some(("-100", ())));
    assert_eq!(iter.next().transpose()?, Some(("100", ())));
    drop(iter);

    Ok(())
}

 */