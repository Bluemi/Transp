use std::env;
mod receiver;
mod sender;
mod packet;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate bincode;

const PORT: u16 = 2345;

pub fn print_usage() {
	println!("transp:");
	println!("");
	println!("transp r [-q]");
	println!("transp s [-q] [<IP> [<FILE>]]");
}

fn main() {
	let mut args = env::args().skip(1);
	match args.next() {
		Some(ref x) if x == "r" => receiver::call(args),
		Some(ref x) if x == "s" => sender::call(args),
		_ => print_usage(),
	}
}
