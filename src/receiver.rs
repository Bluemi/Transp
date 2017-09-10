use std::iter::Iterator;
use std::net::TcpListener;
use std::io::Read;
use packet::Packet;
use bincode;
use local_ip;

use std::fs::{create_dir, File, OpenOptions};
use std::io::{stdin, BufRead, Write};
use std::path::PathBuf;

// handler

use PORT;

fn call_handler<T: Iterator<Item=String>>(mut args: T) -> Result<(), String> {
	if args.next().is_some() {
		return Err(format!("No arguments are required for the receiver!"));
	}

	let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT))
		.map_err(|x| format!("Failed to bind TcpListener: {}", x))?;

	match local_ip::get() {
		Some(ip) => println!("Your IP: {}", ip),
		None => println!("Couldn't find local IP address"),
	}

	let mut socket = listener.accept()
		.map(|(x, _)| x)
		.map_err(|x| format!("TcpListener::accept() failed: {:?}", x))?;

	loop {
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

		match handle_packet(p) {
			PacketInfo::Stop => return Ok(()),
			PacketInfo::Error(x) => return Err(x),
			PacketInfo::Proceed => {},
		}
	}
}

pub fn call<T: Iterator<Item=String>>(args: T) {
	match call_handler(args) {
		Ok(()) => {},
		Err(x) => {
			println!("Error: {}", x);
		}
	}
}

// packet handlers

fn handle_filecreate(path: String, content: Vec<u8>) -> Result<(), String> {
	let pbuf = secure_filename(path)?;
	let mut f = File::create(pbuf)
		.map_err(|x| format!("Failed creating File: {}", x))?;
	f.write_all(&content[..])
		.map_err(|x| format!("Failed writing to File: {}", x))
}

fn handle_fileappend(path: String, content: Vec<u8>) -> Result<(), String> {
	let pbuf = PathBuf::from(path);
	let mut f = OpenOptions::new().append(true).open(pbuf)
		.map_err(|x| format!("Failed opening File in append mode: {}", x))?;
	f.write_all(&content[..])
		.map_err(|x| format!("Failed appending to File: {}", x))
}

fn handle_dircreate(path: String) -> Result<(), String> {
	let pbuf = secure_filename(path)
		.map_err(|x| format!("Failed Securing Filename: {}", x))?;

	create_dir(pbuf)
		.map_err(|x| format!("Failed creating Directory: {}", x))
	
}

enum PacketInfo {
	Stop,
	Proceed,
	Error(String),
}

fn handle_packet(p: Packet) -> PacketInfo {
	let res = match p {
		Packet::FileCreate { path, content } => handle_filecreate(path, content),
		Packet::FileAppend { path, content } => handle_fileappend(path, content),
		Packet::DirectoryCreate { path } => handle_dircreate(path),
		Packet::Done => return PacketInfo::Stop,
	};
	if let Err(x) = res {
		PacketInfo::Error(x)
	} else {
		PacketInfo::Proceed
	}
}

// io

fn secure_filename<T: Into<PathBuf>>(arg: T) -> Result<PathBuf, String> {
	let mut pbuf = arg.into();
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
