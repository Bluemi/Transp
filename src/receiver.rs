use std::iter::Iterator;
use std::net::TcpListener;
use std::io::{Read, Cursor};
use packet::Packet;
use byteorder::{BigEndian, ReadBytesExt};

// handler

use PORT;

fn call_handler<T: Iterator<Item=String>>(mut args: T) -> Result<(), String> {
	if args.next().is_some() {
		return Err(format!("No arguments are required for the receiver!"));
	}

	let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT))
		.map_err(|x| format!("Failed to bind TcpListener: {}", x))?;

	// TODO: get own ip address, print it

	return match listener.accept() {
		Ok((mut socket, _)) => {
			let mut size_buffer: Vec<u8> = vec![0; 8];
			socket.read(&mut size_buffer)
				.map_err(|x| format!("Failed reading size from socket: {}", x))?;
			let size = Cursor::new(size_buffer).read_u64::<BigEndian>()
				.map(|x| x as usize)
				.map_err(|x| format!("Failed converting size_buffer to size: {}", x.to_string()))?;

			let mut packet_buffer: Vec<u8> = vec![0; size];
			socket.read(&mut packet_buffer)
				.map_err(|x| format!("Failed reading packet from socket: {}", x))?;
			let p = Packet::deserialize(&packet_buffer)
				.map_err(|x| format!("Failed deserializing Packet: {}", x))?;
			p.create_files()
		},
		Err(e) => Err(format!("TcpListener::accept() failed: {:?}", e)),
	};
}

pub fn call<T: Iterator<Item=String>>(args: T) {
	match call_handler(args) {
		Ok(()) => {},
		Err(x) => {
			println!("Error: {}", x);
		}
	}
}

// io

use std::fs::{create_dir, metadata, File};
use std::io::{stdin, BufRead, Write};
use std::env::current_dir;
use std::path::PathBuf;

fn secure_filename(mut filename: String) -> Result<String, String> {
	while path_exists(&filename) {
		println!("File: {} already exists. Do you want a new name? Empty String for 'No':", filename);
		let mut s = String::new();
		let stdin = stdin();
		stdin.lock()
			.read_line(&mut s)
			.map_err(|x| format!("Failed reading a line: {}", x))?;
		if s.is_empty() { break; }
		filename = s;
	}
	return Ok(filename);
}

fn path_exists(path: &str) -> bool {
	metadata(path).is_ok()
}

fn build_file(arg: &str, content: &str) -> Result<(), String> {
	let filename: String = secure_filename(arg.to_string())?;

	let mut f = File::create(filename)
		.map_err(|x| format!("Failed creating File: {}", x))?;
	f.write_all(content.as_bytes())
		.map_err(|x| format!("Failed writing to File: {}", x))
}

fn build_dir(arg: &str) -> Result<(), String> {
	let dirname = secure_filename(arg.to_string())
		.map_err(|x| format!("Failed Securing Filename: {}", x))?;
	create_dir(dirname)
		.map_err(|x| format!("Failed creating Directory: {}", x))
}

impl Packet {
	fn create_files_to(&self, pbuf: PathBuf) -> Result<(), String> {
		match self {
			&Packet::File { ref name, ref content } => {
				build_file(name, content)
			},
			&Packet::Directory { ref name, ref packets } => {
				build_dir(name)?;
				for packet in packets {
					let mut new_pbuf = pbuf.clone();
					new_pbuf.push(name);
					packet.create_files_to(new_pbuf)
						.map_err(|x| format!("Packet::create_files_to failed: {}", x))?;
				}
				Ok(())
			},
		}
	}

	fn create_files(&self) -> Result<(), String> {
		let pbuf = current_dir()
			.map_err(|x| format!("Failed detecting current dir: {}", x))?;
		return self.create_files_to(pbuf);
	}
}
