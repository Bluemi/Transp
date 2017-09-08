use std::iter::Iterator;
use std::net::TcpListener;
use std::io::Read;
use packet::Packet;
use bincode;

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
			let size = bincode::deserialize::<u64>(&size_buffer[..])
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

use std::fs::{create_dir, File};
use std::io::{stdin, BufRead, Write};
use std::env::current_dir;
use std::path::PathBuf;

fn secure_filename(mut pbuf: PathBuf) -> Result<PathBuf, String> {
	while pbuf.exists() {
		println!("File: {} already exists. Do you want a new name? Empty String for 'No':", pbuf.to_string_lossy());
		let mut s = String::new();
		let stdin = stdin();
		stdin.lock()
			.read_line(&mut s)
			.map_err(|x| format!("Failed reading a line: {}", x))?;
		if s.is_empty() { break; }
		pbuf.pop();
		pbuf.push(s);
	}
	return Ok(pbuf);
}

fn build_file(pbuf: PathBuf, content: &str) -> Result<(), String> {
	let pbuf = secure_filename(pbuf)?;
	let mut f = File::create(pbuf)
		.map_err(|x| format!("Failed creating File: {}", x))?;
	f.write_all(content.as_bytes())
		.map_err(|x| format!("Failed writing to File: {}", x))
}

fn build_dir(pbuf: PathBuf) -> Result<(), String> {
	let pbuf = secure_filename(pbuf)
		.map_err(|x| format!("Failed Securing Filename: {}", x))?;

	create_dir(pbuf)
		.map_err(|x| format!("Failed creating Directory: {}", x))
}

impl Packet {
	fn create_files_to(&self, mut pbuf: PathBuf) -> Result<(), String> {
		match self {
			&Packet::File { ref name, ref content } => {
				pbuf.push(name);
				build_file(pbuf, content)
			},
			&Packet::Directory { ref name, ref packets } => {
				pbuf.push(name);
				build_dir(pbuf.clone())?;
				for packet in packets {
					packet.create_files_to(pbuf.clone())
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
