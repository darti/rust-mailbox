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

use super::{entry, Entry};
use tokio::io::{AsyncRead, BufReader};
use crate::stream::AsyncLines;

/// `Iterator` over line based entries.
///
/// This is a lower level building block that exposes an event like interface
/// over the line based one.
///
/// It will repeat a sequence of `Entry`s in the following order:
///
/// - `Entry::Begin`, the beginning of an email.
/// - `Entry::Header`, 0 or more header entries.
/// - `Entry::Body`, 0 or more body entries.
/// - `Entry::End`, the end of the email.
///
/// This is then leveraged by `mail::Iter` to expose a more ergonomic API over
/// actual `Mail`s.
pub struct AsyncIter<R: AsyncRead> {
    input: AsyncLines<R>,
    state: State,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum State {
    Begin,
    Header,
    Body,
}

impl<R: AsyncRead + Unpin> AsyncIter<R> {
    /// Create a new `Iterator` from the given input.
    #[inline]
    pub fn new(input: R) -> Self {
        AsyncIter {
            input: super::async_lines(BufReader::new(input)),
            state: State::Begin,
        }
    }

    pub async fn next(&mut self) -> Option<std::io::Result<Entry>> {
        macro_rules! eof {
            ($body:expr) => {
                if let Some(value) = $body {
                    value
                } else {
                    if self.state == State::Body {
                        self.state = State::Begin;
                        return Some(Ok(Entry::End));
                    }

                    return None;
                }
            };
        }

        macro_rules! utf8 {
            ($body:expr) => {
                match $body {
                    Ok(value) => value,

                    Err(_) => {
                        return Some(Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "stream did not contain valid UTF-8",
                        )))
                    }
                }
            };
        }

        loop {
            let (offset, line) = eof!(self.input.next().await).ok()?;

            match self.state {
                State::Begin => {
                    // Parse the beginning and return any errors.
                    let value = entry::Begin::new(utf8!(String::from_utf8(line))).ok()?;
                    self.state = State::Header;

                    return Some(Ok(Entry::Begin(offset, value)));
                }

                State::Header => {
                    // If the line is empty the header section is over.
                    if line.is_empty() {
                        self.state = State::Body;
                        continue;
                    }

                    // There's an escaped line after the beginning.
                    if line[0] == b'>' {
                        continue;
                    }

                    let mut line = line;

                    // Read lines until there are no folded headers.
                    loop {
                        let consumed;

                        if let Ok((_, ref current)) = *eof!(self.input.peek().await) {
                            match current.first() {
                                Some(&b' ') | Some(&b'\t') => {
                                    line.extend_from_slice(current);
                                    consumed = true;
                                }

                                _ => break,
                            }
                        } else {
                            break;
                        }

                        if consumed {
                            self.input.next().await;
                        }
                    }

                    // Parse the header and return any errors.
                    return Some(Ok(Entry::Header(entry::Header::new(line).ok()?)));
                }

                State::Body => {
                    // If the line is empty there's a newline in the content or a new
                    // mail is beginning.
                    if line.is_empty() {
                        if let Ok((_, ref current)) = *eof!(self.input.peek().await) {
                            // Try to parse the beginning, if it parses it's a new mail.
                            if entry::Begin::ranges(current).is_ok() {
                                self.state = State::Begin;
                                return Some(Ok(Entry::End));
                            }
                        }
                    }

                    return Some(Ok(Entry::Body(line)));
                }
            }
        }
    }
}

