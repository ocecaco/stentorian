use std::collections::HashMap;

pub struct Interner<'a> {
    name_to_id: HashMap<&'a str, u32>,
    names: Vec<(u32, &'a str)>,
}

impl<'a> Interner<'a> {
    pub fn new() -> Self {
        Interner {
            name_to_id: HashMap::new(),
            names: Vec::new(),
        }
    }

    pub fn intern(&mut self, s: &'a str) -> u32 {
        if let Some(&id) = self.name_to_id.get(s) {
            id
        } else {
            let id = (self.names.len() + 1) as u32;

            self.names.push((id, s));
            self.name_to_id.insert(s, id);
            id
        }
    }

    pub fn done(self) -> Vec<(u32, &'a str)> {
        self.names
    }
}
