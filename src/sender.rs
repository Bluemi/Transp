use std::iter::Iterator;
use std::net::TcpStream;

use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::convert::From;

use packet::Packet;

// client
pub fn call<T: Iterator<Item=String>>(mut args: T) {
	let ip = args.next()
		.expect("Ip address missing");
	let filename = args.next()
		.expect("Filename missing");

	let connection_string = format!("{}:2345", ip);
	let mut stream = TcpStream::connect(connection_string).unwrap();

	// stream.write(...)
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

impl Packet {
	fn from(filename: &str) -> Result<Packet, String> {
		match File::open(&filename) {
			Ok(mut f) => {
				let mut contents = String::new();
				match f.read_to_string(&mut contents) {
					Ok(_) => return Result::Ok(Packet::File{
								name: String::from(cut_path(filename)),
								content: contents.clone(),
							}),
					Err(_) => return Result::Err(String::from(format!("couldnt read from {}", filename))),
				}
			},
			// Wenn es kein File ist, ist es ein Ordner?
			Err(_) => {
				match fs::read_dir(filename) {
					Ok(entries) => {
						let mut packets : Vec<Packet> = Vec::new();
						for entry in entries {
							match entry {
								Ok(dir) => {
									match Packet::from(dir.path().to_str().unwrap()) {
										Ok(p) => packets.push(p),
										Err(s) => return Result::Err(s),
									}
								},
								Err(_) => return Result::Err(String::from(format!("couldnt read dir {}", filename))),
							}
						}
						return Result::Ok(Packet::Directory{name: String::from(cut_path(filename)), packets: packets});
					},
					Err(_) => return Result::Err(String::from(format!("couldnt read dir {}", filename))),
				}
			},
		}
	}
}
