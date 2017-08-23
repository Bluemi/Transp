use std::iter::Iterator;
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::Read;
use std::str::from_utf8;

pub fn call<T: Iterator<Item=String>>(args: T) {
	println!("receiver called");
	let listener = TcpListener::bind("0.0.0.0:2345").unwrap();
	match listener.accept() {
		Ok((_socket, addr)) => { println!("new client {:?}", addr); read_from_socket(_socket); },
		Err(e) => println!("couldn't get client: {:?}", e),
	};
}

fn read_from_socket(mut stream: TcpStream) {
	let mut buffer = vec![0; 20];
	stream.read(&mut buffer).unwrap();
	println!("message: \"{}\"", from_utf8(&buffer).unwrap());
}
