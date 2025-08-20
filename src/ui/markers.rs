use anathema::geometry::Pos;

static SYMBOLS: &[&str] = &["//", "#", ";;", ";", "--"];

// If a string is less than this many bytes
// it could not possibly hold a marker
const MIN_POSSIBLE_LEN: usize = 3;

pub fn generate(text: impl Into<String>) -> (String, Option<Markers>) {
    let mut markers = vec![];

    let content = text
        .into()
        .split_inclusive('\n')
        .enumerate()
        .filter_map(|(offset, line)| match marker(offset - markers.len(), line) {
            Some(marker) => {
                markers.push(marker);
                None
            }
            None => Some(escape_marker(line)),
        })
        .collect();

    let markers = (!markers.is_empty()).then(|| markers.into());
    (content, markers)
}

// -----------------------------------------------------------------------------
//   - Marker -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Marker {
    pub row: usize,
    name: String,
}

impl From<&Marker> for Pos {
    fn from(marker: &Marker) -> Self {
        Self::new(0, marker.row as i32)
    }
}

// marker: <maybe comment> @<ident>

// Possible comment syntax:
// * //
// * #
// * ;
// * --

// 1. Trim whitespace
// 2. Source[..possible comment symbols length] == possible comment symbol
// 3. Trim whitespace
// 4. Position of '@'
// 5. Marker = line[pos..].take_while(char::is_ascii_alphabetic].join()
fn marker(offset: usize, line: &str) -> Option<Marker> {
    let mut line = line.trim_start();

    if line.len() < MIN_POSSIBLE_LEN {
        return None;
    }

    let symbol_len = SYMBOLS
        .iter()
        .find(|symbol| line.starts_with(*symbol))
        .map(|symbol| symbol.len())?;

    line = line[symbol_len..].trim();

    if line.len() < 2 || line.as_bytes()[0] != b'@' || line.starts_with("@@") {
        return None;
    }

    // Strip the marker prefix: '@'
    line = line[1..].trim();

    let marker = line
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>();

    Some(Marker {
        row: offset,
        name: marker.to_string(),
    })
}

fn escape_marker(input: &str) -> String {
    let line = input.trim_start();

    if line.len() < MIN_POSSIBLE_LEN {
        return input.into();
    }

    let symbol_len = SYMBOLS
        .iter()
        .find(|symbol| line.starts_with(*symbol))
        .map(|symbol| symbol.len());

    let len = match symbol_len {
        None => return input.into(),
        Some(len) => len,
    };

    let diff = input.len() - line.len() + len;
    let line = &input[diff..];

    match line.trim().starts_with("@@") {
        true => {
            let offset = diff + line.len() - line.trim().len();
            let (a, b) = input.split_at(offset);
            let mut buffer = String::with_capacity(input.len() - 1);
            buffer.push_str(a);
            buffer.push_str(&b[1..]);
            buffer
        }
        false => input.into(),
    }
}

// -----------------------------------------------------------------------------
//   - Markers -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Markers {
    inner: Vec<Marker>,
}

impl Markers {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    fn offset_by(&mut self, offset_by: usize) {
        self.inner.iter_mut().for_each(|marker| marker.row += offset_by);
    }

    fn split(&mut self, index: usize) -> Self {
        let new = self.inner.split_off(index);
        Self { inner: new }
    }

    pub fn offset_after(&mut self, row: usize, offset: usize) {
        let index = self.inner.partition_point(|marker| marker.row < row);
        self.inner[index..].iter_mut().for_each(|marker| marker.row += offset);
    }

    pub fn get(&self, key: &str) -> Option<&Marker> {
        self.inner.iter().find(|Marker { name, .. }| key.eq(name))
    }

    // * offset new markers by insertion point
    // * offset current markers *after* the insertion point with N lines
    pub fn merge(&mut self, insert_after_row: usize, mut other: Self) {
        // Offset the new rows by the insertion point (row)
        other.offset_by(insert_after_row);

        // The last marker before the insertion point
        let index = self.inner.partition_point(|marker| marker.row < insert_after_row);

        // Offset the existing markers by the number of new rows added
        let rhs = self.split(index);

        // Add the new markers in before joining the old ones back in
        self.inner.extend(other);
        self.inner.extend(rhs);
    }

    pub(crate) fn clear(&mut self) {
        self.inner.clear();
    }
}

impl From<Vec<Marker>> for Markers {
    fn from(inner: Vec<Marker>) -> Self {
        Self { inner }
    }
}

impl IntoIterator for Markers {
    type IntoIter = <Vec<Marker> as IntoIterator>::IntoIter;
    type Item = Marker;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_markers() {
        let s = "// @zero
a
// @one
b
// @two
c"
        .to_string();

        let (_, markers) = generate(s);
        let markers = markers.unwrap();

        for (i, marker) in markers.inner.into_iter().enumerate() {
            assert_eq!(i, marker.row);
        }
    }

    #[test]
    fn merge_markers() {
        let mut markers = Markers::new();
        markers.inner.push(Marker {
            row: 0,
            name: "B".to_string(),
        });
        markers.inner.push(Marker {
            row: 1,
            name: "C".to_string(),
        });

        let mut other = Markers::new();
        other.inner.push(Marker {
            row: 0,
            name: "A".to_string(),
        });

        // Insert A before B
        markers.merge(0, other);

        assert_eq!(markers.inner[0].row, 0);
        assert_eq!(markers.inner[1].row, 0);
        assert_eq!(markers.inner[2].row, 1);

        assert_eq!(&markers.inner[0].name, "A");
        assert_eq!(&markers.inner[1].name, "B");
        assert_eq!(&markers.inner[2].name, "C");
    }

    #[test]
    fn escape_markers() {
        let input = "  // @@escape";
        let actual = escape_marker(input);
        let expected = "  // @escape";
        assert_eq!(expected, &*actual);
    }
}
