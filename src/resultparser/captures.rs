#[derive(Debug, Clone, Serialize)]
pub struct CaptureTree<'a, T> {
    pub rule: &'a str,
    pub name: &'a str,
    pub slice: T,
    pub children: Vec<CaptureTree<'a, T>>,
}

pub type Match<'a> = CaptureTree<'a, (usize, usize)>;

#[derive(Debug, Copy, Clone)]
enum Capture {
    Started(usize),
    Complete(usize, usize),
}

impl Capture {
    fn complete(&self) -> (usize, usize) {
        if let Capture::Complete(a, b) = *self {
            (a, b)
        } else {
            panic!("attempt to unwrap incomplete capture");
        }
    }
}

fn complete_capture_tree<'a>(tree: &CaptureTree<'a, Capture>) -> Match<'a> {
    let completed_children = tree.children.iter().map(|c| complete_capture_tree(c));

    CaptureTree {
        rule: tree.rule,
        name: tree.name,
        slice: tree.slice.complete(),
        children: completed_children.collect(),
    }
}

#[derive(Debug, Clone)]
pub struct CaptureBuilder<'a> {
    captures: Vec<CaptureTree<'a, Capture>>,
}

impl<'a> CaptureBuilder<'a> {
    pub fn new() -> Self {
        CaptureBuilder {
            captures: Vec::new(),
        }
    }

    pub fn capture_start(&mut self, rule: &'a str, name: &'a str, position: usize) {
        self.captures
            .push(CaptureTree {
                      rule: rule,
                      name: name,
                      slice: Capture::Started(position),
                      children: Vec::new(),
                  });
    }

    pub fn capture_stop(&mut self, position: usize) {
        {
            let mut child = self.captures.last_mut().unwrap();
            if let Capture::Started(start) = child.slice {
                child.slice = Capture::Complete(start, position);
            } else {
                panic!("attempt to stop capture twice");
            }
        }

        if self.captures.len() >= 2 {
            let child = self.captures.pop().unwrap();

            let parent = self.captures.last_mut().unwrap();
            if let Capture::Complete(_, _) = parent.slice {
                panic!("attempt to add child to completed parent");
            }

            parent.children.push(child);
        }
    }

    pub fn done(self) -> Match<'a> {
        complete_capture_tree(&self.captures[0])
    }
}
