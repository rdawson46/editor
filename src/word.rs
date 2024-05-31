use std::char;

#[derive(PartialEq, Eq, Clone, Copy)]
enum CharKind {
    Space,
    Punct,
    Other,
}

impl CharKind {
    fn new(c: char) -> Self{
        if c.is_whitespace() {
            Self::Space
        } else if c.is_ascii_punctuation() {
            Self::Punct
        } else {
            Self::Other
        }
    }
}

pub fn find_word_start_forward(line: &String, start_col: usize) -> Option<usize> {
    let mut it = line.chars().enumerate().skip(start_col);
    let mut prev = CharKind::new(it.next()?.1);
    for (col, c) in it {
        let cur = CharKind::new(c);
        if cur != CharKind::Space && prev != cur {
            return Some(col);
        }
        prev = cur;
    }
    None
}

pub fn find_word_end_forward(line: &String, start_col: usize) -> Option<usize> {
    let mut it = line.chars().enumerate().skip(start_col + 1);
    let mut prev = CharKind::new(it.next()?.1);
    for (col, c) in it {
        let cur = CharKind::new(c);
        if prev != CharKind::Space && prev != cur {
            return Some(col-1);
        }
        prev = cur;
    }
    None
}

pub fn find_word_start_backward(line: &String, start_col: usize) -> Option<usize> {
    let idx = line
        .char_indices()
        .nth(start_col)
        .map(|(i, _)| i)
        .unwrap_or(line.len());
    let mut it = line[..idx].chars().rev().enumerate();
    let mut cur = CharKind::new(it.next()?.1);
    for (i, c) in it {
        let next = CharKind::new(c);
        if cur != CharKind::Space && next != cur {
            return Some(start_col - i);
        }
        cur = next;
    }
    (cur != CharKind::Space).then(|| 0)
}

#[warn(dead_code)]
pub fn find_next_occur_forward (line: &String, start_col: usize, target: char) -> Option<usize> {
    let it = line.chars().enumerate().skip(start_col);
    for (col, c) in it {
        if c == target {
            return Some(col);
        }
    }
    None
}

#[warn(dead_code)]
pub fn find_next_occur_backward (line: &String, start_col: usize, target: char) -> Option<usize> {
    let idx = line
        .char_indices()
        .nth(start_col)
        .map(|(i, _)| i)
        .unwrap_or(line.len());
    let it = line[..idx].chars().rev().enumerate();
    for (col, c) in it {
        if c == target {
            return Some(col);
        }
    }
    None
}
