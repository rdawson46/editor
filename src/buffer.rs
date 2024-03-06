use std::path::{Path, PathBuf};
use color_eyre::eyre::Result;
use std::fs::{File, read_dir};
use std::io::{BufReader, BufRead};

enum BufferType {
    Empty,
    Directory,
    File
}

pub struct Line{
    pub text: Box<String>,
    pub length: u16
}

// fields will be added later
pub struct Lines{
    pub lines: Vec<Line>
}

pub struct Buffer<'a> {
    b_type: BufferType,
    pub path: Option<&'a Path>,
    pub lines: Lines,
    pub ptr_y: u16,
    pub ptr_x: u16,
}

impl Buffer<'_> {
    pub fn new(path: &Path) -> Result<Buffer> {
        // TODO: handle unkown paths better
        let btype: BufferType;
        let lines: Lines;

        if path.exists() {
            if path.is_file() {
                let file = File::open(&path);

                if let Ok(file) = file {
                    let reader = BufReader::new(file);
                    btype = BufferType::File;

                    let mut file_lines: Vec<Line> = vec![];

                    for line in reader.lines() {
                        match line {
                            Ok(text) => {
                                let length: u16 = text.len().try_into().unwrap();
                                let boxed_text = Box::new(text);
                                file_lines.push( Line {
                                    text: boxed_text,
                                    length
                                });
                            },
                            Err(_) => {}
                        }
                    }

                    lines = Lines { lines: file_lines };
                } else {
                    panic!("couldn't open file")
                }

            } else if path.is_dir() {
                btype = BufferType::Directory;
                let mut entries = vec![];
                lines = Lines { lines: vec![] };

                let reader = read_dir(path).unwrap();


                // TODO: sort entries and handle events for dirs
                for path in reader {
                    entries.push(path.unwrap().file_name());
                }
            } else {
                panic!("what did you do");
            }
        } else {
            btype = BufferType::Empty;
            lines = Lines { lines: vec![] }
        }

        return Ok(Buffer {
            b_type: btype,
            path: Some(path),
            lines,
            ptr_x: 0,
            ptr_y: 0,
        });
    }
}
