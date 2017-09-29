use std::path::PathBuf;
use std::io::Write;
use std::fs::{File, OpenOptions, create_dir};

use packet::Packet;

use receiver::allow::is_allowed;

pub enum PacketInfo {
	Stop,
	Proceed,
	Error(String),
}

pub fn handle_packet(p: Packet) -> PacketInfo {
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

fn handle_filecreate(path: String, content: Vec<u8>) -> Result<(), String> {
	let pbuf = PathBuf::from(path);
	if !is_allowed(&pbuf)? {
		return Err(format!("Creating File is not allowed: {:?}", pbuf));
	}
	let mut f = File::create(pbuf)
		.map_err(|x| format!("Failed creating File: {}", x))?;
	f.write_all(&content[..])
		.map_err(|x| format!("Failed writing to File: {}", x))
}

fn handle_fileappend(path: String, content: Vec<u8>) -> Result<(), String> {
	let pbuf = PathBuf::from(path);
	if !is_allowed(&pbuf)? {
		return Err(format!("Appending to File is not allowed: {:?}", pbuf));
	}
	let mut f = OpenOptions::new().append(true).open(pbuf)
		.map_err(|x| format!("Failed opening File in append mode: {}", x))?;
	f.write_all(&content[..])
		.map_err(|x| format!("Failed appending to File: {}", x))
}

fn handle_dircreate(path: String) -> Result<(), String> {
	let pbuf = PathBuf::from(path);
	if !is_allowed(&pbuf)? {
		return Err(format!("Creating Directory is not allowed: {:?}", pbuf));
	}

	create_dir(pbuf)
		.map_err(|x| format!("Failed creating Directory: {}", x))
}
