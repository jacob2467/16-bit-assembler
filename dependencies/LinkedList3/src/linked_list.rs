use std::fmt;
use crate::node::Node;

#[derive(Debug)]
pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    len: usize
}

impl<T: Clone + PartialEq> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList{head: None, len: 0}
    }

    pub fn iter(&self) -> Iter<T> {
        if let Some(head) = &self.head {
            Iter{next_node: Some(&**head)}
        } else {
            Iter{next_node: None}
        }
    }

    fn head_as_ref(&self) -> Option<&Node<T>> {
        if let Some(node) = &self.head {
            return Some(&**node);
        }
        None
    }

    fn head_as_mut(&mut self) -> Option<&mut Node<T>> {
        if let Some(node) = &mut self.head {
            return Some(&mut **node);
        }
        None
    }

    pub fn add(&mut self, value: T) {
        let newnode: Option<Box<Node<T>>> = Node::new_wrapped(value, self.head.take());
        self.len += 1;
        self.head = newnode;
    }

    pub fn append(&mut self, value: T) {
        let newnode: Option<Box<Node<T>>> = Node::new_wrapped(value, None);
        self.len += 1;
        if self.is_empty() {
            self.head = newnode;
        } else if let Some(lastnode) = self.last_mut() {
            lastnode.set_next(newnode);
        }
    }

    pub fn find(&self, value: &T) -> Option<usize> {
        let mut i: usize = 0;
        while i < self.len() as usize {
            let node = self.node_at(i)?;
            if node.value == *value {
                return Some(i);
            }
            i += 1;
        }
        None
    }
    
    pub fn replace(&mut self, index: usize, mut value: T) {
        let old_node = self.mut_node_at(index).unwrap();
        std::mem::swap(&mut old_node.value, &mut value)
    }
    
    fn mut_node_at(&mut self, index: usize) -> Option<&mut Node<T>> {
        let mut current_index = 0;
        let mut current = self.head_as_mut();
        while current_index < index {
            current = current.take()?.next_mut();
            current_index += 1;
        };
        current
    }

    fn node_at(&self, index: usize) -> Option<&Node<T>> {
        let mut current_index = 0;
        let mut current = self.head_as_ref();
        while current_index < index {
            current = current.take()?.peek_next();
            current_index += 1;
        };
        current
    }

    pub fn peek(&self) -> Option<&T> {
        Some(self.head.as_ref()?.value())
    }

    pub fn peek_back(&mut self) -> Option<&T> {
        match self.last_mut() {
            Some(node) => Some(node.value()),
            None => None
        }
    }

    pub fn remove(&mut self) -> Option<T> {
        let to_return: Option<T> = self.peek().cloned();

        self.head = match self.head.take() {
            Some(mut head) => {
                self.len -= 1;
                head.take_next()
            },
            None => None
        };
        to_return
    }

    fn last_mut(&mut self) -> Option<&mut Node<T>> {
        let mut current: Option<&mut Node<T>> = self.head_as_mut();
        while let Some(node) = current {
            if node.peek_next().is_none() {
                return node.as_mut_node();
            }
            current = node.next_mut();
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.head = None;
    }

    pub fn len(&self) -> usize {
        self.len
    }
}


impl<T: Clone + PartialEq + ToString> LinkedList<T> {
    pub fn build_string(&self) -> String {
        // String builder
        let mut output: String = String::from("Linked List: {");
        let mut first = true;

        let mut iter = self.iter();
        while let Some(next) = iter.next() {
            if ! first {
                output.push_str(", ");
            }
            first = false;
            output.push_str(&next.to_string());
        }

        output.push_str("}");
        output
    }
}

impl<T: Clone + PartialEq + ToString> fmt::Display for LinkedList<T> {
    /// Formats the list as "Linked List: {value1, value2, value3}"
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.build_string())
    }
}

impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        Self{head: self.head.clone(), len: self.len}
    }
}

#[derive(Debug)]
pub struct Iter<'a, T> {
    next_node: Option<&'a Node<T>>
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_node.map(|node| {
            self.next_node = node.peek_next();
            node.value()
        })
    }
}