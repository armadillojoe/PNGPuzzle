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
use std::string::String;
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
		if args[1] == "inject" {
			if args.len() < 4 {
				println!("Usage {} encode <src> <dest>", args[0]);
			} else {
				let src_path = Path::new(&args[3]);
				let mut src_file = File::open(&src_path).unwrap();
				inject_payload(&dest_file, &src_file, &args[3]);
			}
		} else if args[1] == "view" {
			read_header(&dest_file);
			read_chunk(&dest_file, true);
		} else if args[1] == "extract" {
			read_header(&dest_file);
			extract_payload(&dest_file);
		}
	}
}

// Reads the eight byte header of a png file
fn read_header(mut file: &File) {
	let mut buf = [0u8; 8];
	file.read(&mut buf).unwrap();
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
	let chunk_buf = buf.clone();
	let chunk_name = str::from_utf8(&chunk_buf).unwrap();
	let is_end = chunk_name == "IEND";

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

// Extracts the data hidden in a paLD chunk and writes it
// to a new file
fn extract_payload(mut file: &File) {
	// Gets size of chunk
	let size = file.read_u32::<BigEndian>().unwrap();
	
	// Check to see if this is the paLD chunk
	let mut buf = [0u8; 4];
	file.read(&mut buf).unwrap();
	let chunk_buf = buf.clone();
	let chunk_name = str::from_utf8(&chunk_buf).unwrap();
	
	// If not the paLD chunk, move to next chunk and check,
	// or else extract the payload
	if chunk_name != "paLD" {
		file.seek(SeekFrom::Current((size + 4) as i64)).unwrap();
		extract_payload(&file);
	} else {
		// Set up payload file
		let name_length = file.read_u16::<BigEndian>().unwrap();
		let mut filename_vec = vec![0u8; name_length as usize];
		file.read(&mut filename_vec).unwrap();
		let filename = String::from_utf8(filename_vec).unwrap();
		let payload = Path::new(&filename);
		let mut payload_file = File::create(&payload).unwrap();
		
		// Read the payload from the original file
		let mut buf = vec![0u8; size as usize];
		file.read(&mut buf).unwrap();
		
		// Write to payload file
		payload_file.write_all(&mut buf).unwrap();
	}
}

// Creates a new png that copies the file src, into the
// file dest. Data encoded as a paLD chunk
fn inject_payload(mut dest: &File, mut src: &File, filename: &String) {
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
	
	// Write the length of the hidden files name as a u16 
	// and the name itsself to the payload
	let filename_bytes = filename.as_bytes();
	let length = filename_bytes.len() as u16;
	payload.write_u16::<BigEndian>(length).unwrap();
	payload.write_all(&filename_bytes).unwrap();
	
	// Write the hidden file to the payload buffer
	payload.write_all(&mut src_buf).unwrap();
	
	// Create a crc32 checksum over the payload data
	let checksum = crc32::checksum_ieee(payload.as_slice());
	
	// Write all data to the new file, src is now hidden in dest
	out_file.write_all(&mut dest_buf).unwrap();
	out_file.write_u32::<BigEndian>((src_size + (length as usize) + 2) as u32).unwrap();
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