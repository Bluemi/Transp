use std::env;
mod receiver;
mod sender;

fn print_usage() {
	println!("transp:");
	println!("");
	println!("transp -r");
	println!("transp -s IP FILE");
}

fn main() {
	let mut arguments = env::args().skip(1);
	if let Some(x) = arguments.next() {
		match &*x {
			"-r" => receiver::call(arguments),
			"-s" => sender::call(arguments),
			_ => print_usage(),
		}
	} else {
		print_usage();
	}
}
