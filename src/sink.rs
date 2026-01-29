use std::io::{Write, Result};

pub struct Sink;

trait IsEmoji {
    fn is_emoji(&self) -> bool;
}
impl Write for Sink {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // Claim to have successfully written the whole buffer.
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl IsEmoji for Sink {
    fn is_emoji(&self) -> bool {
        true
    }
}

#[test]
pub fn test() {
    let mut out = Sink;
    out.write_all(b"hello world\n");
    println!("{:}", out.is_emoji());
}