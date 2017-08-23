use std::iter::Iterator;
use std::net::TcpStream;

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
