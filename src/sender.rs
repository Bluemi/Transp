use std::iter::Iterator;
use std::net::TcpStream;

use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::convert::From;

use std::path::PathBuf;

use packet::Packet;
use bincode;

use PORT;

pub fn call<T: Iterator<Item=String>>(mut args: T) {
	let ip = args.next()
		.expect("Ip address missing");
	let filename = args.next()
		.expect("Filename missing");

	let connection_string = format!("{}:{}", ip, PORT);
	let mut stream = TcpStream::connect(connection_string).unwrap();

	send(&PathBuf::from(&filename), &mut stream).unwrap();

}

fn send(path: &PathBuf, stream: &mut TcpStream) -> Result<(), String> {
	let open_str: &str = path.to_str()
		.ok_or_else(|| String::from("cant transform path into string"))?;
	let mut file = File::open(open_str)
		.map_err(|x| x.to_string())?;

	let metad = file.metadata()
		.map_err(|x| x.to_string())?;
	if metad.is_dir() {
		send_dir(stream, path)
	} else {
		send_file(&mut file, stream, path)
	}
}

fn send_dir(stream: &mut TcpStream, path: &PathBuf) -> Result<(), String> {
	let path_str: &str = path.to_str()
		.ok_or_else(|| String::from("cant transform path into string"))?;
	let p: Packet = Packet::DirectoryCreate {
		path: String::from(path_str),
	};
	send_packet(&p, stream).unwrap();

	// send recursive sub directories/files
	let entries = fs::read_dir(path)
		.map_err(|x| x.to_string())?;
	for entry in entries {
		let dir = entry.map_err(|x| x.to_string())?;
		let mut tmp_path = path.clone();
		tmp_path.push(dir.file_name());
		send(&tmp_path, stream).unwrap();
	}
	Ok(())
}

fn send_file(file: &mut File, stream: &mut TcpStream, path: &PathBuf) -> Result<(), String> {
	let mut contents : Vec<u8> = Vec::new();
	match file.read_to_end(&mut contents) {
		Ok(_) => {
			let path_str: &str = path.to_str()
				.ok_or_else(|| String::from("cant transform path into string"))?;
			let packet: Packet = Packet::FileCreate {
				path: String::from(path_str),
				content: contents,
			};
			send_packet(&packet, stream).unwrap();
		},
		Err(err) => {
			let path_str: &str = path.to_str()
				.ok_or_else(|| String::from("cant transform path into string"))?;
			return Err(format!("couldnt read from \"{}\": {:?}", path_str, err))
		}
	}
	Ok(())
}

fn send_packet(packet: &Packet, stream: &mut TcpStream) -> Result<(), String> {
	let arr = packet.serialize().map_err(|x| x.to_string())?;
	let len_vec = bincode::serialize(&(arr.len() as u64), bincode::Infinite).map_err(|x| x.to_string())?;
	if let Err(err) = stream.write(&len_vec) {
		println!("failed to write in stream! {}", err.to_string());
	}
	if let Err(err) = stream.write(&arr) {
		println!("failed to write in stream! {}", err.to_string());
	}
	Ok(())
}

impl Packet {
/*
	fn from_dir(filename: &str) -> Result<Packet, String> {
		let entries = fs::read_dir(filename)
			.map_err(|x| x.to_string())?;
		let mut packets : Vec<Packet> = Vec::new();
		for entry in entries {
			let dir = entry.map_err(|x| x.to_string())?;
			match Packet::from(dir.path().to_str().unwrap()) {
				Ok(p) => packets.push(p),
				Err(s) => return Err(s),
			}
		}
		return Ok(Packet::Directory{name: String::from(cut_path(filename)), packets: packets});
	}
*/
}
