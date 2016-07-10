/*
 * Copyright 2016 Joseph Jimenez
 * Program that hides data in a png file
 */
extern crate byteorder;
extern crate crc;

use std::io::prelude::*;
use std::io::SeekFrom;
use std::fs::File;
use std::path::Path;
use std::str;
use std::env;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crc::crc32;

#[allow(unused_mut)]
fn main() {
	let args: Vec<_> = env::args().collect();
	if args.len() < 3 {
		println!("Usage {} <mode> <args>", args[0]);
	} else {
		let path = Path::new(&args[2]);
		let mut dest_file = File::open(&path).unwrap();
		if args[1] == "encode" {
			if args.len() < 4 {
				println!("Usage {} encode <src> <dest>", args[0]);
			} else {
				let src_path = Path::new(&args[3]);
				let mut src_file = File::open(&src_path).unwrap();
				inject_payload(&dest_file, &src_file);
			}
		} else if args[1] == "view" {
			read_header(&dest_file);
			read_chunk(&dest_file, true);
		}
	}
}

fn read_header(mut file: &File) {
	let mut buf = [0u8; 8];
	file.read(&mut buf).unwrap();
	println!("Reading header...");
	for i in 0..8 {
		print!("{:02x} ", buf[i]);
	}
	print!("\n\n");
}

fn read_chunk(mut file: &File, read_all: bool) {
	let size = file.read_u32::<BigEndian>().unwrap();
	println!("Chunk size: {}", size);
	let mut buf = [0u8; 4];
	file.read(&mut buf).unwrap();
	print!("Chunk type: ");
	for i in 0..4 {
		print!("{}", buf[i] as char);
	}
	print!("\n");
	let end = [73, 69, 78, 68];
	let mut is_end = true;
	for i in 0..4 {
		if buf[i] != end[i] {
			is_end = false;
			break;
		}
	}
	file.seek(SeekFrom::Current(size as i64)).unwrap();
	file.read(&mut buf).unwrap();
	print!("Checksum: ");
	for i in 0..4 {
		print!("{:02x}", buf[i]);
	}
	print!("\n\n");
	if is_end {
		return;
	}
	if read_all {
		read_chunk(&file, read_all);
	}
}

fn inject_payload(mut dest: &File, mut src: &File) {
	let out = Path::new("out.png");
	let mut out_file = File::create(&out).unwrap();
	let dest_size = dest.metadata().unwrap().len() as usize;
	let mut dest_buf = vec![0u8; dest_size - 12];
	let meta = src.metadata().unwrap();
	let src_size = meta.len() as usize;
	let mut payload = vec![];
	let mut temp = vec![0u8; src_size];
	dest.read(&mut dest_buf).unwrap();
	src.read(&mut temp).unwrap();
	payload.push(112);
	payload.push(97);
	payload.push(76);
	payload.push(68);
	for i in 0..temp.len() {
		payload.push(temp[i]);
	}
	let checksum = crc32::checksum_ieee(payload.as_slice());
	out_file.write_all(&mut dest_buf).unwrap();
	out_file.write_u32::<BigEndian>(src_size as u32).unwrap();
	out_file.write_all(&mut payload).unwrap();
	out_file.write_u32::<BigEndian>(checksum).unwrap();
	write_end(&out_file);
}

fn write_end(mut file: &File) {
	file.write_u32::<BigEndian>(0).unwrap();
	file.write_all("IEND".as_bytes()).unwrap();
	file.write_u32::<BigEndian>(2923585666).unwrap();
}
