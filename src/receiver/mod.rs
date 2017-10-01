use std::iter::Iterator;

mod network;
use self::network::{open_connection, fetch_packet};

mod ip;
use self::ip::dump_ip;

mod allow;

mod handler;
use self::handler::{PacketInfo, handle_packet};
use ::print_usage;

fn call_handler<T: Iterator<Item=String>>(args: T) -> Result<(), String> {
	let mut quiet = false;

	for x in args {
		if x == "-q" { quiet = true; }
		else {
			print_usage();
			return Ok(());
		}
	}

	if !quiet { dump_ip(); }

	let mut stream = open_connection()?;

	let mut last_file = None;
	loop {
		let packet = fetch_packet(&mut stream)?;
		match handle_packet(packet, quiet, &mut last_file) {
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

