use std::iter::Iterator;
use std::net::TcpListener;
use std::io::Read;
use packet::Packet;

// handler

fn call_handler<T: Iterator<Item=String>>(mut args: T) -> Result<(), String> {
	if args.next().is_some() {
		return Err(format!("No arguments are required for the receiver!"));
	}

	let listener = TcpListener::bind("0.0.0.0:2345")
		.map_err(|x| x.to_string())?;

	// TODO: get own ip address, print it

	return match listener.accept() {
		Ok((mut socket, _)) => {
			let mut buffer = Vec::new(); // vec![...]
			socket.read(&mut buffer)
				.map_err(|x| x.to_string())?;
			let p = Packet::deserialize(&buffer)?;
			p.create_files()
		},
		Err(e) => Err(format!("couldn't get client: {:?}", e)),
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
			.map_err(|x| x.to_string())?;
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
		.map_err(|x| x.to_string())?;
	f.write_all(content.as_bytes())
		.map_err(|x| x.to_string())
}

fn build_dir(arg: &str) -> Result<(), String> {
	let dirname = secure_filename(arg.to_string())?;
	create_dir(dirname)
		.map_err(|x| x.to_string())
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
					packet.create_files_to(new_pbuf)?;
				}
				Ok(())
			},
		}
	}

	fn create_files(&self) -> Result<(), String> {
		let pbuf = current_dir()
			.map_err(|x| x.to_string())?;
		return self.create_files_to(pbuf);
	}
}
