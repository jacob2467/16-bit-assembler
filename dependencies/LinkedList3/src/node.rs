#[derive(Debug)]
pub struct Node<T> {
    pub value: T,
    pub next: Option<Box<Node<T>>>
}

impl<T: Clone> Clone for Node<T> {
    fn clone(&self) -> Self {
        let next: Option<Box<Node<T>>>;
        match self.next {
            Some(_) => next = self.next.clone(),
            None => next = None
        };
        let value = self.value.clone();
        Self::new(value, next)
    }
}

impl<T> Node<T> {
    pub fn new(value: T, next: Option<Box<Node<T>>>) -> Self {
        Node {value, next}
    }

    pub fn new_wrapped(value: T, next: Option<Box<Node<T>>>) -> Option<Box<Node<T>>> {
        Some(Box::new(Node {value, next}))
    }

    pub fn value(&self) -> &T {
        &self.value
    }
    
    pub fn mut_value(&mut self) -> &mut T {
        &mut self.value
    }
    
    pub fn as_ref(&self) -> Option<&Self> {
        Some(&self)
    }

    pub fn as_mut_node(&mut self) -> Option<&mut Self> {
        Some(self)
    }

    pub fn peek_next(&self) -> Option<&Node<T>> {
        if let Some(next) = &self.next {
            return Some(&**next)
        }
        None
    }

    pub fn next_mut(&mut self) -> Option<&mut Node<T>> {
        if let Some(next) = self.next.as_mut() {
            return Some(next)
        }
        None
    }

    pub fn set_next(&mut self, new_next: Option<Box<Node<T>>>) {
        let _old_next: Option<Box<Node<T>>> = self.next.take();
        self.next = new_next;
    }

    pub fn take_next(&mut self) -> Option<Box<Node<T>>> {
        self.next.take()
    }
}