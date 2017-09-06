use std::iter::Iterator;
use std::net::TcpStream;

use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::convert::From;

use packet::Packet;
use byteorder::BigEndian;
use byteorder::WriteBytesExt;

use PORT;

// client
pub fn call<T: Iterator<Item=String>>(mut args: T) {
	let ip = args.next()
		.expect("Ip address missing");
	let filename = args.next()
		.expect("Filename missing");

	let connection_string = format!("{}:{}", ip, PORT);
	let mut stream = TcpStream::connect(connection_string).unwrap();

	let res = Packet::from(&filename);
	if let Ok(packet) = res {
		if let Ok(arr) = packet.serialize() {
			let mut len_vec: Vec<u8> = Vec::new();
			if let Err(err) = len_vec.write_u64::<BigEndian>(arr.len() as u64) {
				println!("cant convert arr.len to byte array: {:?}", err);
			}
			if let Err(err) = stream.write(&len_vec) {
				println!("failed to write in stream! {}", err.to_string());
			}
			if let Err(err) = stream.write(&arr) {
				println!("failed to write in stream! {}", err.to_string());
			}
		}
	} else {
		println!("not ok");
	}
}

fn cut_path(path: &str) -> &str {
	path.split('/').last().unwrap_or(path)
}

#[test]
fn test_cut_path()
{
	let path = String::from("this/is/a/path.txt");
	let s = cut_path(&path);
	assert_eq!(s, "path.txt");
}

#[test]
fn test_cut_path2()
{
	let path = String::from("this/is/a/dir/");
	let s = cut_path(&path);
	assert_eq!(s, "dir");
}

impl Packet {
	fn from(filename: &str) -> Result<Packet, String> {
		let file = File::open(filename)
			.map_err(|x| x.to_string())?;

		let metad = file.metadata()
			.map_err(|x| x.to_string())?;
		if metad.is_dir() {
			return Packet::from_dir(filename);
		} else {
			return Packet::from_file(file, filename);
		}
	}

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

	fn from_file(mut file: File, filename: &str) -> Result<Packet, String> {
		let mut contents = String::new();
		match file.read_to_string(&mut contents) {
			Ok(_) => return Ok(Packet::File{
						name: String::from(cut_path(filename)),
						content: contents.clone(),
					}),
			Err(_) => return Err(String::from(format!("couldnt read from \"{}\"", filename))),
		}
	}
}
