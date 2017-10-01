use std::iter::Iterator;
use std::net::TcpStream;

use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::convert::From;
use std::io;

use std::path::PathBuf;

use packet::Packet;
use bincode;

use PORT;

const MAX_CONTENT_SIZE : usize = 1024 * 1024;

fn call_handler<T: Iterator<Item=String>>(args: T) -> Result<(), String> {
	let (quiets, other_args): (Vec<String>, Vec<String>) = args.partition(|x| x == "-q");

	let quiet = !quiets.is_empty();

	let mut iter = other_args.into_iter();

	let ip = read_cli(iter.next(), "IP-Address")?;
	let filename = read_cli(iter.next(), "Filename")?;

	let connection_string = format!("{}:{}", ip, PORT);
	let mut stream = TcpStream::connect(connection_string).unwrap();

	let filename_pathbuf = PathBuf::from(&filename).canonicalize().map_err(|_| String::from("cant canonicalize filename"))?;
	let filename_path = filename_pathbuf.file_name().ok_or_else(|| format!("cant get filename from {}", &filename))?;
	let send_path = PathBuf::from(filename_path);

	send(&PathBuf::from(&filename), &send_path, &mut stream, quiet)?;
	send_packet(&Packet::Done, &mut stream)?;
	Ok(())
}

pub fn call<T: Iterator<Item=String>>(args: T) {
	if let Err(err) = call_handler(args) {
		println!("{}", err);
	}
}

fn read_cli(arg: Option<String>, name: &str) -> Result<String, String> {
	if let Some(x) = arg { return Ok(x); }
	print!("Enter {}: ", name);
	io::stdout().flush().unwrap();
	let mut input = String::new();
	io::stdin().read_line(&mut input)
		.map_err(|x| x.to_string())?;
	input.pop(); // remove \n
	return Ok(input);
}

fn send(read_path: &PathBuf, send_path: &PathBuf, stream: &mut TcpStream, quiet: bool) -> Result<(), String> {
	let open_str: &str = read_path.to_str()
		.ok_or_else(|| String::from("cant transform path into string"))?;
	let mut file = File::open(open_str)
		.map_err(|x| x.to_string())?;

	let metad = file.metadata()
		.map_err(|x| x.to_string())?;
	if metad.is_dir() {
		send_dir(stream, read_path, send_path, quiet)
	} else {
		send_file(&mut file, stream, read_path, send_path, quiet)
	}
}

fn send_dir(stream: &mut TcpStream, read_path: &PathBuf, send_path: &PathBuf, quiet: bool) -> Result<(), String> {
	let send_path_str: &str = send_path.to_str()
		.ok_or_else(|| String::from("cant transform path into string"))?;
	if !quiet { println!("sending dir:  {}", send_path_str); }
	let p: Packet = Packet::DirectoryCreate {
		path: String::from(send_path_str),
	};
	send_packet(&p, stream).unwrap();

	// send recursive sub directories/files
	let entries = fs::read_dir(read_path)
		.map_err(|x| x.to_string())?;
	for entry in entries {
		let dir = entry.map_err(|x| x.to_string())?;
		let mut tmp_read_path = read_path.clone();
		tmp_read_path.push(dir.file_name());
		let mut tmp_send_path = send_path.clone();
		tmp_send_path.push(dir.file_name());
		send(&tmp_read_path, &tmp_send_path, stream, quiet).unwrap(); // !!!!!!!
	}
	Ok(())
}

fn send_file(file: &mut File, stream: &mut TcpStream, read_path: &PathBuf, send_path: &PathBuf, quiet: bool) -> Result<(), String> {
	if !quiet {
		println!("sending file: {}", send_path.to_str().ok_or_else(|| String::from("cant convert send_path into string"))?);
	}
	let mut file_completely_sent : bool = false;
	let mut file_started : bool = true;
	while !file_completely_sent {
		let mut contents : Vec<u8> = vec![0; MAX_CONTENT_SIZE];
		match file.read(&mut contents) {
			Ok(send_size) => {
				if send_size != MAX_CONTENT_SIZE {
					file_completely_sent = true;
					contents.truncate(send_size);
				}
				let path_str: &str = send_path.to_str()
					.ok_or_else(|| String::from("cant transform path into string"))?;
				if file_started {
					let packet: Packet = Packet::FileCreate {
						path: String::from(path_str),
						content: contents,
					};
					send_packet(&packet, stream).unwrap();
				} else {
					let packet: Packet = Packet::FileAppend {
						path: String::from(path_str),
						content: contents,
					};
					send_packet(&packet, stream).unwrap();
				}
				file_started = false;
			},
			Err(err) => {
				let path_str: &str = read_path.to_str()
					.ok_or_else(|| String::from("cant transform path into string"))?;
				return Err(format!("couldnt read from \"{}\": {:?}", path_str, err))
			}
		}
	}
	Ok(())
}

fn send_packet(packet: &Packet, stream: &mut TcpStream) -> Result<(), String> {
	let arr = packet.serialize().map_err(|x| x.to_string())?;
	let len_vec = bincode::serialize(&(arr.len() as u64), bincode::Infinite).map_err(|x| x.to_string())?;
	if let Err(err) = stream.write(&len_vec) {
		return Err(format!("failed to write len in stream! {}", err.to_string()));
	}
	if let Err(err) = stream.write(&arr) {
		return Err(format!("failed to write in stream! {}", err.to_string()));
	}
	Ok(())
}
