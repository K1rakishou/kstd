use std::fmt::Debug;
use kbox::KBox;

mod kbox;

#[derive(Debug)]
struct LinkedList<T> {
    head: Option<KBox<Node<T>>>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        return Self { head: None };
    }

    pub fn push(&mut self, data: T) {
        let mut node = KBox::new(Node::new(data));
        node.next = self.head.take();
        self.head = Some(node);
    }

    pub fn pop(&mut self) -> Option<T> {
        if let Some(mut head) = self.head.take() {
            let next = head.next.take();
            self.head = next;
            return Some(head.into_inner().data);
        }

        return None;
    }
}

impl<T: Debug> LinkedList<T> {
    pub fn debug_print_node_values(&self) {
        let mut current_node: &Option<KBox<Node<T>>> = &self.head;
        let mut node_counter = 0;

        while let Some(node) = current_node {
            let node_ptr = node;

            if node_counter > 0 {
                print!(" -> ");
            }
            print!("{:?}", (*node_ptr).data);

            current_node = &(*node_ptr).next;
            node_counter += 1;
        }

        println!();
    }
}

#[derive(Debug)]
struct Node<T> {
    data: T,
    next: Option<KBox<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(data: T) -> Self {
        return Self { data, next: None };
    }
}

fn main() {
    let mut list: LinkedList<i32> = LinkedList::new();
    println!("pushing 1");
    list.push(1);
    println!("pushing 2");
    list.push(2);    
    println!("pushing 3");
    list.push(3);    
    println!("pushing 4");
    list.push(4);    
    println!("pushing 5");
    list.push(5);    
    println!("pushing 6");
    list.push(6);    
    list.debug_print_node_values();

    println!("{}", list.pop().unwrap());
    list.debug_print_node_values();
    println!("{}", list.pop().unwrap());
    list.debug_print_node_values();
    println!("{}", list.pop().unwrap());
    list.debug_print_node_values();
    println!("{}", list.pop().unwrap());
    list.debug_print_node_values();
    println!("{}", list.pop().unwrap());
    list.debug_print_node_values();
    println!("{}", list.pop().unwrap());
    list.debug_print_node_values();
}
