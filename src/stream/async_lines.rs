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


use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

/// Iterator over ASCII lines.
///
/// The content of a line is not assumed to be in any specific encoding.
pub struct AsyncLines<R: AsyncRead>(BufReader<R>, u64);

impl<R: AsyncRead + Unpin> AsyncLines<R> {
    /// Create a new `Iterator` from the given input.
    #[inline]
    pub fn new(input: R) -> Self {
        AsyncLines(BufReader::new(input), 0)
    }

    #[inline]
    pub async fn next(&mut self) -> Option<std::io::Result<(u64, Vec<u8>)>> {
        let mut buffer = Vec::new();
        let offset = self.1;

        match self.0.read_until(b'\n', &mut buffer).await {
            Ok(0) => None,

            Ok(_) => {
                self.1 += buffer.len() as u64;

                if buffer.last() == Some(&b'\n') {
                    buffer.pop();

                    if buffer.last() == Some(&b'\r') {
                        buffer.pop();
                    }
                }

                Some(Ok((offset, buffer)))
            }

            Err(e) => Some(Err(e)),
        }
    }
}

