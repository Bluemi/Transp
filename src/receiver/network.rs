use std::io::Read;
use std::net::{TcpStream, TcpListener};

use bincode;

use PORT;
use packet::Packet;

pub fn open_connection() -> Result<TcpStream, String> {
	let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT))
		.map_err(|x| format!("Failed to bind TcpListener: {}", x))?;

	listener.accept()
		.map(|(x, _)| x)
		.map_err(|x| format!("TcpListener::accept() failed: {:?}", x))
}

pub fn fetch_packet(socket: &mut TcpStream) -> Result<Packet, String> {
	let mut size_buffer: Vec<u8> = vec![0; 8];
	socket.read_exact(&mut size_buffer)
		.map_err(|x| format!("Failed reading size from socket: {}", x))?;
	let size = bincode::deserialize::<u64>(&size_buffer[..])
		.map(|x| x as usize)
		.map_err(|x| format!("Failed converting size_buffer to size: {}", x.to_string()))?;

	let mut packet_buffer: Vec<u8> = vec![0; size];
	socket.read_exact(&mut packet_buffer)
		.map_err(|x| format!("Failed reading packet from socket: {}", x))?;
	 Packet::deserialize(&packet_buffer)
		.map_err(|x| format!("Failed deserializing Packet: {}", x))
}
