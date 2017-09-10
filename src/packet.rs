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
	}
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

/*
#[test]
fn test_packet_serialization() {
	let p1 = Packet::File {
		name: String::from("NAME"),
		content: String::from("CONTENT"),
	};
	let b = p1.serialize().unwrap();
	let p2 = Packet::deserialize(&b).unwrap();
	match p2 {
		Packet::File { name: n, content: c } => {
			assert_eq!(n, String::from("NAME"));
			assert_eq!(c, String::from("CONTENT"));
		},
		Packet::Directory { .. } => panic!("p2 should be File"),
	}
}

#[test]
fn test_packet_serialization_dir() {
	let f1 = Packet::File {
		name: String::from("NAME"),
		content: String::from("CONTENT"),
	};
	let dir = Packet::Directory {
		name: String::from("thisDir"),
		packets: vec![f1],
	};
	let b = dir.serialize().unwrap();
	let dir2 = Packet::deserialize(&b).unwrap();

	match dir2 {
		Packet::File { .. }  => panic!("dir2 should be a Directory"),
		Packet::Directory{name: n, packets: p} => {
			assert_eq!(String::from("thisDir"), n);
			assert_eq!(1, p.len());
			match &p[0] {
				&Packet::File { name: ref n, content: ref c } => {
					assert_eq!(n, "NAME");
					assert_eq!(c, "CONTENT");
				},
				&Packet::Directory { .. } => panic!("p should be File"),
			}
		}
	}
}
*/
