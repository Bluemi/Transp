use std::net::UdpSocket;

use bincode;

use PORT;
use packet::Packet;

pub fn open_connection() -> Result<UdpSocket, String> {
	UdpSocket::bind(format!("0.0.0.0:{}", PORT))
		.map_err(|x| format!("Failed to bind UdpSocket: {}", x))
}

fn fetch_sized(socket: &mut UdpSocket, size: usize) -> Result<Vec<u8>, String> {
	let mut buffer = vec![0; size];
	let (recv_size, _) = socket.recv_from(&mut buffer)
		.map_err(|x| format!("Failed reading from socket: {}", x))?;

	if recv_size != size { return Err(format!("received size wasn't '{}'", size)); }
	Ok(buffer)
}

pub fn fetch_packet(socket: &mut UdpSocket) -> Result<Packet, String> {
	let size_buffer = fetch_sized(socket, 8)?;

	let size = bincode::deserialize::<u64>(&size_buffer[..])
		.map(|x| x as usize)
		.map_err(|x| format!("Failed converting size_buffer to size: {}", x.to_string()))?;

	let packet_buffer = fetch_sized(socket, size)?;
	Packet::deserialize(&packet_buffer)
		.map_err(|x| format!("Failed deserializing Packet: {}", x))
}
