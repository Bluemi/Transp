use bincode;

#[derive(Serialize, Deserialize)]
pub enum Packet {
	FileCreate {
		path: String,
		content: Vec<u8>,
	},
	FileAppend {
		path: String,
		content: Vec<u8>,
	},
	DirectoryCreate {
		path: String,
	},
	Done
}

impl Packet {
	pub fn serialize(&self) -> Result<Vec<u8>, String> {
		bincode::serialize(self, bincode::Infinite)
			.map_err(|x| x.to_string())
	}

	pub fn deserialize(bin: &Vec<u8>) -> Result<Packet, String> {
		bincode::deserialize(&bin[..])
			.map_err(|x| x.to_string())
	}
}

#[test]
fn test_packet_serialization_filecreate() {
	let obj = Packet::FileCreate {
		path: String::from("PATH"),
		content: vec![1, 2, 4],
	};
	let bytes = obj.serialize().unwrap();
	match Packet::deserialize(&bytes).unwrap() {
		Packet::FileCreate { path: p, content: c } => {
			assert_eq!(p, String::from("PATH"));
			assert_eq!(c, vec![1, 2, 4]);
		},
		_ => panic!("should be FileCreate"),
	}
}

#[test]
fn test_packet_serialization_fileappend() {
	let obj = Packet::FileAppend {
		path: String::from("PATH"),
		content: vec![2, 3, 5],
	};
	let bytes = obj.serialize().unwrap();
	match Packet::deserialize(&bytes).unwrap() {
		Packet::FileAppend { path: p, content: c } => {
			assert_eq!(p, String::from("PATH"));
			assert_eq!(c, vec![2, 3, 5]);
		},
		_ => panic!("should be FileAppend"),
	}
}

#[test]
fn test_packet_serialization_dircreate() {
	let obj = Packet::DirectoryCreate {
		path: String::from("PATH"),
	};
	let bytes = obj.serialize().unwrap();
	match Packet::deserialize(&bytes).unwrap() {
		Packet::DirectoryCreate { path: p } => {
			assert_eq!(String::from("PATH"), p);
		},
		_ => panic!("should be DirectoryCreate"),
	}
}

#[test]
fn test_packet_serialization_done() {
	let obj = Packet::Done;
	let bytes = obj.serialize().unwrap();
	match Packet::deserialize(&bytes).unwrap() {
		Packet::Done => {},
		_ => panic!("should be DirectoryCreate"),
	}
}
