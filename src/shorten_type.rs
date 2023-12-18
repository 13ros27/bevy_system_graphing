use std::borrow::Cow;

use bevy::{ecs::schedule::NodeId, utils::HashMap};

#[derive(Debug)]
pub struct FunctionName {
    sections: Vec<Vec<String>>, // Sections (things like <>), split into blocks (::) internally
}

impl FunctionName {
    // pub fn shorten_str(full_name: &str) -> String {
    //     if let Some(before) = full_name.strip_suffix("::{{closure}}") {
    //         return format!("{}::{{{{closure}}}}", Self::shorten_str(before));
    //     }

    //     let mut short_name = String::new();
    //     for path in full_name.split_inclusive(&['<', '>', '(', ')', '[', ']', ',', ';'][..]) {
    //         // Push the shortened path in front of the found character
    //         short_name.push_str(path.rsplit(':').next().unwrap().trim());
    //     }

    //     short_name
    // }

    pub fn new(full_name: &str) -> FunctionName {
        if let Some(before) = full_name.strip_suffix("::{{closure}}") {
            FunctionName::new(before)
        } else {
            FunctionName {
                sections: full_name
                    .split_inclusive(&['<', '>', '(', ')', '[', ']', ',', ';'][..])
                    .map(|s| {
                        s.trim()
                            .split("::")
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>()
                    })
                    .collect(),
            }
        }
    }

    fn shortest(&self) -> String {
        let mut short_name = String::new();
        for sect in &self.sections {
            short_name.push_str(sect.last().unwrap());
        }
        short_name
    }
}

// Just a test of some algos
pub fn shorten_pair(func1: FunctionName, func2: FunctionName) -> (String, String) {
    let mut func1name = String::new();
    let mut func2name = String::new();
    for (sect1, sect2) in func1.sections.into_iter().zip(func2.sections) {
        if sect1 != sect2 {
            let mut block = Vec::new();
            for (block1, block2) in sect1.into_iter().rev().zip(sect2.into_iter().rev()) {
                if block1 == block2 {
                    block.push(block1);
                } else {
                    block.reverse();
                    let block_part = &(String::from("::") + &block.join("::"));
                    func1name.push_str(&(block1 + block_part));
                    func2name.push_str(&(block2 + block_part));
                }
            }
        } else {
            // These are the same so just get the last item
            func1name.push_str(sect1.last().unwrap());
            func2name.push_str(sect2.last().unwrap());
        }

        if func1name.chars().last() == Some(',') {
            func1name.push(' ');
        }
        if func2name.chars().last() == Some(',') {
            func2name.push(' ');
        }
    }

    (func1name, func2name)
}

// TODO: Handle ambiguity
pub fn shorten_systems(systems: HashMap<NodeId, Cow<str>>) -> HashMap<NodeId, String> {
    systems
        .iter()
        .map(|(&n, s)| (n, FunctionName::new(s).shortest()))
        .collect()
}
