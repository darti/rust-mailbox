//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.

use std::io;
use stream::entry::header;
use util::Address;
use super::Header;

pub struct ReplyTo {
	address: Address,
}

impl Header for ReplyTo {
	#[inline]
	fn name() -> &'static str {
		"Reply-To"
	}

	#[inline]
	fn parse(values: &[header::Item]) -> io::Result<Self> {
		Ok(ReplyTo {
			address: try!(Address::new(values[0].clone()))
		})
	}
}

impl ReplyTo {
	#[inline]
	pub fn address(&self) -> &Address {
		&self.address
	}
}
