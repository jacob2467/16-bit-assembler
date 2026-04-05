#![allow(dead_code)]
#[allow(unused_imports)]
use crate::linked_list::LinkedList;
mod node;
mod linked_list;

fn main() {
    let mut ll: LinkedList<&str> = LinkedList::new();
    ll.add("three");
    ll.add("two");
    ll.add("one");

    let mut ll2 = ll.clone();
    println!("ll1: {}", ll);
    
    ll2.append("four");
    ll2.append("five");
    ll2.append("six");
    println!("ll2: {}", ll2);
}
