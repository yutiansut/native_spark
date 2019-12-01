#![allow(where_clauses_object_safety)]
#[macro_use]
extern crate serde_closure;
#[macro_use]
extern crate itertools;
use chrono::prelude::*;
use native_spark::*;
use parquet::column::reader::get_typed_column_reader;
use parquet::data_type::{ByteArrayType, Int32Type, Int64Type};
use parquet::file::reader::{FileReader, SerializedFileReader};

use std::fs;
use std::fs::File;
use std::path::Path;

fn main() -> Result<()> {
    let sc = Context::new()?;
    let files = fs::read_dir("parquet_file_dir")
        .unwrap()
        .map(|x| x.unwrap().path().to_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let len = files.len();
    let files = sc.make_rdd(files, len);
    let read = files.flat_map(Fn!(|file| read(file)));
    let sum = read.reduce_by_key(Fn!(|((vl, cl), (vr, cr))| (vl + vr, cl + cr)), 1);
    let avg = sum.map(Fn!(|(k, (v, c))| (k, v as f64 / c)));
    let res = avg.collect().unwrap();
    println!("{:?}", &res[0]);
    Ok(())
}

fn read(file: String) -> Box<dyn Iterator<Item = ((i32, String, i64), (i64, f64))>> {
    let file = File::open(&Path::new(&file)).unwrap();
    let reader = SerializedFileReader::new(file).unwrap();
    let metadata = reader.metadata();
    let batch_size = 500_000 as usize;
    //let reader = Rc::new(RefCell::new(reader));
    let iter = (0..metadata.num_row_groups()).flat_map(move |i| {
        //let reader = reader.borrow_mut();
        let row_group_reader = reader.get_row_group(i).unwrap();
        let mut first_reader =
            get_typed_column_reader::<Int32Type>(row_group_reader.get_column_reader(0).unwrap());
        let mut second_reader = get_typed_column_reader::<ByteArrayType>(
            row_group_reader.get_column_reader(1).unwrap(),
        );
        let mut bytes_reader =
            get_typed_column_reader::<Int64Type>(row_group_reader.get_column_reader(7).unwrap());
        let mut time_reader =
            get_typed_column_reader::<Int64Type>(row_group_reader.get_column_reader(8).unwrap());
        let num_rows = metadata.row_group(i).num_rows() as usize;
        println!("row group rows {}", num_rows);
        let mut chunks = vec![];
        let mut batch_count = 0 as usize;
        while batch_count < num_rows {
            let begin = batch_count;
            let mut end = batch_count + batch_size;
            if end > num_rows {
                end = num_rows as usize;
            }
            chunks.push((begin, end));
            batch_count = end;
        }
        println!("total rows-{} chunks-{:?}", num_rows, chunks);
        chunks.into_iter().flat_map(move |(begin, end)| {
            let end = end as usize;
            let begin = begin as usize;
            let mut first = vec![Default::default(); end - begin];
            let mut second = vec![Default::default(); end - begin];
            let mut time = vec![Default::default(); end - begin];
            let mut bytes = vec![Default::default(); end - begin];
            first_reader
                .read_batch(batch_size, None, None, &mut first)
                .unwrap();
            second_reader
                .read_batch(batch_size, None, None, &mut second)
                .unwrap();
            time_reader
                .read_batch(batch_size, None, None, &mut time)
                .unwrap();
            bytes_reader
                .read_batch(batch_size, None, None, &mut bytes)
                .unwrap();
            let first = first.into_iter();
            let second = second
                .into_iter()
                .map(|x| unsafe { String::from_utf8_unchecked(x.data().to_vec()) });
            let time = time.into_iter().map(|t| {
                let t = t / 1000;
                i64::from(Utc.timestamp(t, 0).hour())
            });
            let bytes = bytes.into_iter().map(|b| (b, 1.0));
            let key = izip!(first, second, time);
            let value = bytes;
            key.zip(value)
        })
    });
    Box::new(iter)
}
