use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

pub struct Store<T: Eq + Hash> {
    forward: Vec<Rc<T>>,
    backward: HashMap<Rc<T>, usize>
}

impl<T: Eq + Hash> Store<T> {
    pub fn new() -> Self {
        Self {
            forward: Vec::new(),
            backward: HashMap::new()
        }
    }

    pub fn add(&mut self, item: T) -> Option<usize> {
        let id = self.forward.len();

        let rc = Rc::new(item);
        if self.backward.insert(rc.clone(), id).is_none() {
            self.forward.push(rc);
            Some(id)
        } else {
            None
        }
    }
    
    pub fn get(&self, id: usize) -> Option<&T> {
        self.forward.get(id).map(|rc| rc.as_ref())
    }
}
