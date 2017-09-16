extern crate get_if_addrs;

pub fn dump_ip() {
	match get_if_addrs::get_if_addrs() {
		Ok(ifaces) => {
			match ifaces.iter()
				.map(|x| x.ip())
				.map(|x| x.to_string())
				.filter(|x| x.starts_with("192.168.178."))
				.next() {

				Some(ip) => println!("Your IP: {}", ip),
				None => println!("Couldn't find local IP address"),
			}
		},
		Err(_) => println!("Couldn't access IP addresses"),
	}
}
