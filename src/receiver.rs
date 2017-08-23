use std::iter::Iterator;

pub fn call<T: Iterator<Item=String>>(args: T) {
	println!("receiver called with args:");
}
