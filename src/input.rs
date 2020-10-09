use crate::Result;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug)]
pub struct Input {
    hosts: Vec<String>,
}

impl Input {
    pub fn new(path: Option<&str>) -> Self {
        let hosts = Self::read(path).expect("unable to read input");
        Self { hosts }
    }

    fn read(path: Option<&str>) -> Result<Vec<String>> {
        let mut contents = Vec::new();
        let reader: Box<dyn BufRead> = match path {
            Some(filepath) => {
                Box::new(BufReader::new(File::open(filepath).map_err(|e| {
                    format!("tried to read file {} got {}", filepath, e)
                })?))
            }
            None => Box::new(BufReader::new(io::stdin())),
        };

        for line in reader.lines() {
            contents.push(line?)
        }

        Ok(contents)
    }

    pub fn hosts(self) -> Vec<String> {
        self.hosts
    }
}
