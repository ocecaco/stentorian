use std::collections::HashMap;

pub struct Interner<'a, U> {
    name_to_id: HashMap<&'a str, U>,
    names: Vec<(U, &'a str)>,
}

impl<'a, U: From<u32> + Into<u32> + Copy> Interner<'a, U> {
    pub fn new() -> Self {
        Interner {
            name_to_id: HashMap::new(),
            names: Vec::new(),
        }
    }

    pub fn intern(&mut self, s: &'a str) -> U {
        if let Some(&id) = self.name_to_id.get(s) {
            id
        } else {
            let id = U::from((self.names.len() + 1) as u32);

            self.names.push((id, s));
            self.name_to_id.insert(s, id);
            id
        }
    }

    pub fn done(self) -> Vec<(u32, &'a str)> {
        self.names.iter()
            .map(|&(id, s)| (U::into(id), s))
            .collect()
    }
}
