use std::path::PathBuf;

// TODO is_allowed should be prevent overwriting files.

pub fn is_allowed(p: &PathBuf) -> Result<bool, String> {
	let p = normalize(p)?;
	let wd: PathBuf = normalize(&get_wd()?)
		.map_err(|x| format!("Failed to check allowance: {}", x))?;
	Ok(p != wd && p.starts_with(wd))
}

fn get_wd() -> Result<PathBuf, String> {
	use std::env::current_dir;

	current_dir()
		.map_err(|x| format!("Failed to detect current dir: {}", x))
}

fn normalize(pbuf: &PathBuf) -> Result<PathBuf, String> {
	use std::path::Component;

	let mut components = pbuf.components();

	let mut out_pbuf = match components.next() {
		Some(Component::Prefix(x)) => PathBuf::from(x.as_os_str()),
		Some(Component::RootDir) => PathBuf::from(Component::RootDir.as_os_str()),
		Some(Component::Normal(x)) => get_wd()?.join(x),
		Some(_) => get_wd()?,
		None => { return Err(format!("Failed to normalize empty path!")); },
	};

	for component in components {
		match component {
			Component::CurDir => {},
			Component::ParentDir => { out_pbuf.pop(); }
			Component::Normal(x) => { out_pbuf.push(x); },
			comp => { return Err(format!("Normalizing Failed: Initial-Component within path: {:?}, component is: {:?}", &pbuf, comp)); }
		}
	}

	Ok(out_pbuf)
}

#[test]
fn test_normalize1() {
	assert_eq!(normalize(&PathBuf::from("/ok")).unwrap(), PathBuf::from("/ok"));
}

#[test]
fn test_normalize2() {
	assert_eq!(normalize(&PathBuf::from("/ok/foo/./bar/..")).unwrap(), PathBuf::from("/ok/foo"));
}

#[test]
fn test_normalize3() {
	assert_eq!(normalize(&PathBuf::from("/")).unwrap(), PathBuf::from("/"));
}
