static TAB: &str = "    ";

// -----------------------------------------------------------------------------
//   - Text buffer -
// -----------------------------------------------------------------------------
/// This is text that should be typed out by the editor
pub struct TextBuffer {
    inner: String,
    index: usize,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            inner: String::new(),
            index: 0,
        }
    }

    pub fn push(&mut self, s: impl AsRef<str>) {
        self.inner.push_str(s.as_ref());
    }

    pub fn next(&mut self) -> Option<&str> {
        if self.index == self.inner.len() {
            self.index = 0;
            self.inner.clear();
            return None;
        }

        let next = &self.inner[self.index..];
        if next.starts_with(TAB) {
            self.index += TAB.len();
            return Some(TAB);
        }

        let next_index = next.chars().next()?.len_utf8();
        let retval = &next[..next_index];

        self.index += next_index;

        Some(retval)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn buffer_next() {
        let mut buf = TextBuffer::new();
        buf.push("a    🐇bc");

        assert_eq!("a", buf.next().unwrap());
        assert_eq!("    ", buf.next().unwrap());
        assert_eq!("🐇", buf.next().unwrap());
        assert_eq!("b", buf.next().unwrap());
        assert_eq!("c", buf.next().unwrap());
        assert!(buf.next().is_none());
    }
}
