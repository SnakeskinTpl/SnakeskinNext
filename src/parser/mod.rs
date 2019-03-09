pub mod node;
use self::node::*;

use std::fs::File;
use std::io::{BufReader, Error, BufRead};
use std::path::{Path, Iter};

use std::convert::AsRef;
use std::iter::Iterator;
use std::string::*;

pub fn parse_file<T: AsRef<Path>>(path: T) -> Result<Vec<Node>, Error> {
    let file = File::open(path)?;
    let iter = BufReader::new(file)
        .lines()
        .filter(|val| val.is_ok())
        .map(|val| val.unwrap());

    Ok(parse(iter))
}

pub fn parse<T: Iterator<Item=String>>(iter: T) -> Vec<Node> {
    let mut res = Vec::new();

    'lines: for (line, content) in iter.enumerate() {
        let mut bytes = content.bytes().peekable();

        while let Some(cursor) = bytes.peek() {
            let is_whitespace = cursor.is_ascii_whitespace();

            if cursor.is_ascii_punctuation() {
                let el = cursor.clone();
                bytes.next();

                if let Some(cursor) = bytes.peek() {
                    if cursor.is_ascii_punctuation() {
                        let expr = unsafe {
                            String::from_utf8_unchecked(vec![el, cursor.clone()])
                        };

                        match expr.as_ref() {
                            "//" => {
                                res.push(Node::SingleComment {
                                    line: line as u32 + 1,
                                    content
                                });

                                continue 'lines;
                            },

                            _ => {}
                        }

                        println!("{:?}", expr);
                    }
                }

            } else {
                bytes.next();
            }

            if !is_whitespace {
                continue 'lines;
            }
        }
    }

    println!("{:?}", res);
    res
}
