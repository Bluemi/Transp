use std::iter::Iterator;

mod network;
use self::network::{open_connection, fetch_packet};

mod ip;
use self::ip::dump_ip;

mod allow;

mod handler;
use self::handler::{PacketInfo, handle_packet};

fn call_handler<T: Iterator<Item=String>>(mut args: T) -> Result<(), String> {
	if args.next().is_some() {
		return Err(format!("No arguments are required for the receiver!"));
	}

	let mut stream = open_connection()?;
	dump_ip();

	println!("Starting to receive data!");

	loop {
		let packet = fetch_packet(&mut stream)?;
		match handle_packet(packet) {
			PacketInfo::Proceed => continue,
			PacketInfo::Stop => return Ok(()),
			PacketInfo::Error(x) => return Err(x),
		}
	}
}

pub fn call<T: Iterator<Item=String>>(args: T) {
	if let Err(err) = call_handler(args) {
		println!("Error: {}", err);
	}
}

