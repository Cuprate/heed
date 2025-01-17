use std::error::Error;
use std::fs;
use std::path::Path;

use heed::byteorder::BE;
use heed::types::*;
use heed::{Database, EnvOpenOptions};
use serde::{Deserialize, Serialize};

fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new("target").join("heed.mdb");

    fs::create_dir_all(&path)?;

    let env = EnvOpenOptions::new()
        .map_size(10 * 1024 * 1024) // 10MB
        .max_dbs(3000)
        .open(path)?;

    // here the key will be an str and the data will be a slice of u8
    let mut wtxn = env.write_txn()?;
    let db: Database<Str, Bytes> = env.create_database(&mut wtxn, Some("kiki"))?;

    db.put(&mut wtxn, "hello", &[2, 3][..])?;
    let ret: Option<&[u8]> = db.get(&wtxn, "hello")?;

    println!("{:?}", ret);
    wtxn.commit()?;

    // serde types are also supported!!!
    #[derive(Debug, Serialize, Deserialize)]
    struct Hello<'a> {
        string: &'a str,
    }

    let mut wtxn = env.write_txn()?;
    let db: Database<Str, SerdeBincode<Hello>> =
        env.create_database(&mut wtxn, Some("serde-bincode"))?;

    let hello = Hello { string: "hi" };
    db.put(&mut wtxn, "hello", &hello)?;

    let ret: Option<Hello> = db.get(&wtxn, "hello")?;
    println!("serde-bincode:\t{:?}", ret);

    wtxn.commit()?;

    let mut wtxn = env.write_txn()?;
    let db: Database<Str, SerdeJson<Hello>> = env.create_database(&mut wtxn, Some("serde-json"))?;

    let hello = Hello { string: "hi" };
    db.put(&mut wtxn, "hello", &hello)?;

    let ret: Option<Hello> = db.get(&wtxn, "hello")?;
    println!("serde-json:\t{:?}", ret);

    wtxn.commit()?;

    // you can ignore the data
    let mut wtxn = env.write_txn()?;
    let db: Database<Str, Unit> = env.create_database(&mut wtxn, Some("ignored-data"))?;

    db.put(&mut wtxn, "hello", &())?;
    let ret: Option<()> = db.get(&wtxn, "hello")?;

    println!("{:?}", ret);

    let ret: Option<()> = db.get(&wtxn, "non-existant")?;

    println!("{:?}", ret);
    wtxn.commit()?;

    // database opening and types are tested in a safe way
    //
    // we try to open a database twice with the same types
    let mut wtxn = env.write_txn()?;
    let _db: Database<Str, Unit> = env.create_database(&mut wtxn, Some("ignored-data"))?;

    // and here we try to open it with other types
    // asserting that it correctly returns an error
    //
    // NOTE that those types are not saved upon runs and
    // therefore types cannot be checked upon different runs,
    // the first database opening fix the types for this run.
    let result = env.create_database::<Str, SerdeJson<i32>>(&mut wtxn, Some("ignored-data"));
    assert!(result.is_err());

    // you can iterate over keys in order
    type BEI64 = I64<BE>;

    let db: Database<BEI64, Unit> = env.create_database(&mut wtxn, Some("big-endian-iter"))?;

    db.put(&mut wtxn, &0, &())?;
    db.put(&mut wtxn, &68, &())?;
    db.put(&mut wtxn, &35, &())?;
    db.put(&mut wtxn, &42, &())?;

    let rets: Result<Vec<(i64, _)>, _> = db.iter(&wtxn)?.collect();

    println!("{:?}", rets);

    // or iterate over ranges too!!!
    let range = 35..=42;
    let rets: Result<Vec<(i64, _)>, _> = db.range(&wtxn, &range)?.collect();

    println!("{:?}", rets);

    // delete a range of key
    let range = 35..=42;
    let deleted: usize = db.delete_range(&mut wtxn, &range)?;

    let rets: Result<Vec<(i64, _)>, _> = db.iter(&wtxn)?.collect();

    println!("deleted: {:?}, {:?}", deleted, rets);
    wtxn.commit()?;

    Ok(())
}
