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

// Reads the eight byte header of a png file
fn read_header(mut file: &File) {
	let mut buf = [0u8; 8];
	file.read(&mut buf).unwrap();
	println!("Reading header...");
	for i in 0..8 {
		print!("{:02x} ", buf[i]);
	}
	print!("\n\n");
}

// Reads all the different chunks of the png, outputing
// chunk size, type, and the checksum. If read_all is true
// funciton works recursively, reading chunks until the last.
fn read_chunk(mut file: &File, read_all: bool) {
	// Gets size of chunk
	let size = file.read_u32::<BigEndian>().unwrap();
	println!("Chunk size: {}", size);
	
	// Gets type of chunk
	let mut buf = [0u8; 4];
	file.read(&mut buf).unwrap();
	print!("Chunk type: ");
	for i in 0..4 {
		print!("{}", buf[i] as char);
	}
	print!("\n");
	
	// Check to see if this is end chunk
	let end = [73, 69, 78, 68];
	let mut is_end = true;
	for i in 0..4 {
		if buf[i] != end[i] {
			is_end = false;
			break;
		}
	}
	
	// Get the checksum
	file.seek(SeekFrom::Current(size as i64)).unwrap();
	file.read(&mut buf).unwrap();
	print!("Checksum: ");
	for i in 0..4 {
		print!("{:02x}", buf[i]);
	}
	print!("\n\n");
	
	// Exit is end chunk seen
	if is_end {
		return;
	}
	
	// Read next chunk if recursive
	if read_all {
		read_chunk(&file, read_all);
	}
}

// Creates a new png that copies the file src, into the
// file dest. Data encoded as a paLD chunk
fn inject_payload(mut dest: &File, mut src: &File) {
	// Ready the output file
	let out = Path::new("out.png");
	let mut out_file = File::create(&out).unwrap();
	
	// Create buffer to read in the dest file
	let dest_size = dest.metadata().unwrap().len() as usize;
	let mut dest_buf = vec![0u8; dest_size - 12];
	dest.read(&mut dest_buf).unwrap();
	
	// Create buffer to read in the src file to hide
	let src_size = src.metadata().unwrap().len() as usize;
	let mut src_buf = vec![0u8; src_size];
	src.read(&mut src_buf).unwrap();
	
	// Ready the payload that will contain paLD chunk type
	// and the src file data
	let mut payload = vec![];
	payload.push(112);  // p
	payload.push(97);  // a
	payload.push(76);  // L
	payload.push(68);  //D
	for i in 0..src_buf.len() {
		payload.push(src_buf[i]);
	}
	
	// Create a crc32 checksum over the payload data
	let checksum = crc32::checksum_ieee(payload.as_slice());
	
	// Write all data to the new file, src is now hidden in dest
	out_file.write_all(&mut dest_buf).unwrap();
	out_file.write_u32::<BigEndian>(src_size as u32).unwrap();
	out_file.write_all(&mut payload).unwrap();
	out_file.write_u32::<BigEndian>(checksum).unwrap();
	write_end(&out_file);
}

// Writes the ending chunk to the file
fn write_end(mut file: &File) {
	file.write_u32::<BigEndian>(0).unwrap();
	file.write_all("IEND".as_bytes()).unwrap();
	file.write_u32::<BigEndian>(2923585666).unwrap();
}