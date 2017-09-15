use std::iter::Iterator;
use std::net::TcpListener;
use std::io::{Read, Write};
use std::fs::{create_dir, File, OpenOptions};
use std::path::PathBuf;

use bincode;
extern crate get_if_addrs;

use packet::Packet;

// handler

use PORT;

fn call_handler<T: Iterator<Item=String>>(mut args: T) -> Result<(), String> {
	if args.next().is_some() {
		return Err(format!("No arguments are required for the receiver!"));
	}

	let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT))
		.map_err(|x| format!("Failed to bind TcpListener: {}", x))?;

	match get_if_addrs::get_if_addrs() {
		Ok(ifaces) => {
			match ifaces.iter()
				.map(|x| x.ip())
				.map(|x| x.to_string())
				.filter(|x| x.starts_with("192.168.178."))
				.next() {

				Some(ip) => println!("Your IP: {}", ip),
				None => println!("Couldn't find local IP address"),
			}
		},
		Err(_) => println!("Couldn't access IP addresses"),
	}

	let mut socket = listener.accept()
		.map(|(x, _)| x)
		.map_err(|x| format!("TcpListener::accept() failed: {:?}", x))?;

	println!("Starting to receive data!");

	loop {
		let mut size_buffer: Vec<u8> = vec![0; 8];
		socket.read_exact(&mut size_buffer)
			.map_err(|x| format!("Failed reading size from socket: {}", x))?;
		let size = bincode::deserialize::<u64>(&size_buffer[..])
			.map(|x| x as usize)
			.map_err(|x| format!("Failed converting size_buffer to size: {}", x.to_string()))?;

		let mut packet_buffer: Vec<u8> = vec![0; size];
		socket.read_exact(&mut packet_buffer)
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
	let pbuf = PathBuf::from(path);
	if !check_allowance(&pbuf)? {
		return Err(format!("Creating File is not allowed: {:?}", pbuf));
	}
	let mut f = File::create(pbuf)
		.map_err(|x| format!("Failed creating File: {}", x))?;
	f.write_all(&content[..])
		.map_err(|x| format!("Failed writing to File: {}", x))
}

fn handle_fileappend(path: String, content: Vec<u8>) -> Result<(), String> {
	let pbuf = PathBuf::from(path);
	if !check_allowance(&pbuf)? {
		return Err(format!("Appending to File is not allowed: {:?}", pbuf));
	}
	let mut f = OpenOptions::new().append(true).open(pbuf)
		.map_err(|x| format!("Failed opening File in append mode: {}", x))?;
	f.write_all(&content[..])
		.map_err(|x| format!("Failed appending to File: {}", x))
}

fn handle_dircreate(path: String) -> Result<(), String> {
	let pbuf = PathBuf::from(path);
	if !check_allowance(&pbuf)? {
		return Err(format!("Creating Directory is not allowed: {:?}", pbuf));
	}

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

fn check_allowance(p: &PathBuf) -> Result<bool, String> {
	let p = normalize(p)?;
	let wd: PathBuf = get_wd()
		.and_then(|x| normalize(&x))
		.map_err(|x| format!("Failed to check allowance: {}", x))?;
	Ok(p != wd && p.starts_with(wd))
}

fn get_wd() -> Result<PathBuf, String> {
	use std::env::current_dir;

	current_dir()
		.map_err(|x| format!("Couldn't determine working directory: {}", x))
}

fn normalize(pbuf: &PathBuf) -> Result<PathBuf, String> {
	use std::path::Component;

	let mut components = pbuf.components();

	let mut out_pbuf = match components.next() {
		Some(Component::Prefix(x)) => PathBuf::from(x.as_os_str()),
		Some(Component::RootDir) => PathBuf::from(Component::RootDir.as_os_str()),
		Some(Component::Normal(x)) => get_wd()?.join(x),
		Some(_) => get_wd()?,
		None => { return Err(format!("Failed to normalize empty path!")); },
	};

	for component in components {
		match component {
			Component::CurDir => {},
			Component::ParentDir => { out_pbuf.pop(); }
			Component::Normal(x) => { out_pbuf.push(x); },
			comp => { return Err(format!("Normalizing Failed: Initial-Component within path: {:?}, component is: {:?}", &pbuf, comp)); }
		}
	}

	Ok(out_pbuf)
}

#[test]
fn test_normalize1() {
	assert_eq!(normalize(&PathBuf::from("/ok")).unwrap(), PathBuf::from("/ok"));
}

#[test]
fn test_normalize2() {
	assert_eq!(normalize(&PathBuf::from("/ok/foo/./bar/..")).unwrap(), PathBuf::from("/ok/foo"));
}

#[test]
fn test_normalize3() {
	assert_eq!(normalize(&PathBuf::from("/")).unwrap(), PathBuf::from("/"));
}
