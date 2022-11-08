use crate::Type;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct MethodDesc {
	pub parameters: Vec<Type>,
	pub ret: Option<Type>,
}

impl MethodDesc {
	pub fn parse(desc: &str) -> Option<MethodDesc> {
		let end = desc.find(')')?;
		let mut remaining = &desc[1..end];
		let mut parameters = Vec::new();

		let ret_desc = &desc[end + 1..];
		let ret = if ret_desc.as_bytes()[0] == b'V' {
			None
		} else {
			Some(Type::parse(ret_desc)?)
		};

		while !remaining.is_empty() {
			let (parameter, size) = Type::parse_len(remaining)?;
			parameters.push(parameter);
			remaining = &remaining[size..];
		}

		Some(MethodDesc { parameters, ret })
	}
}
