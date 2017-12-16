#[derive(Debug, Clone, Serialize)]
pub struct CaptureTree<'a, T> {
    pub name: &'a str,
    pub slice: T,
    pub children: Vec<CaptureTree<'a, T>>,
}

pub type Match<'a> = CaptureTree<'a, (usize, usize)>;

#[derive(Debug, Copy, Clone)]
enum Capture {
    Started(usize),
    Stopped(usize, usize),
}

impl Capture {
    fn complete(&self) -> (usize, usize) {
        if let Capture::Stopped(a, b) = *self {
            (a, b)
        } else {
            panic!("attempt to unwrap incomplete capture");
        }
    }
}

fn complete_capture_tree<'a>(tree: &CaptureTree<'a, Capture>) -> Match<'a> {
    let completed_children = tree.children.iter().map(|c| complete_capture_tree(c));

    CaptureTree {
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

    pub fn capture_start(&mut self, name: &'a str, position: usize) {
        self.captures
            .push(CaptureTree {
                      name: name,
                      slice: Capture::Started(position),
                      children: Vec::new(),
                  });
    }

    pub fn capture_stop(&mut self, position: usize) {
        {
            let child = self.captures.last_mut().unwrap();

            if let Capture::Started(start) = child.slice {
                child.slice = Capture::Stopped(start, position);
            } else {
                panic!("attempt to stop capture twice");
            }
        }

        if self.captures.len() < 2 {
            return;
        }

        let child = self.captures.pop().unwrap();

        {
            let parent = self.captures.last_mut().unwrap();
            if let Capture::Started(_) = parent.slice {
                parent.children.push(child);
                return;
            }
        }

        self.captures.push(child);
    }

    pub fn done(self) -> Vec<Match<'a>> {
        let children = self.captures.iter().map(|c| complete_capture_tree(c)).collect();

        children
    }
}
