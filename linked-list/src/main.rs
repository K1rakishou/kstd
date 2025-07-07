use std::fmt::Debug;
use std::{ptr::NonNull};

#[derive(Debug)]
struct LinkedList<T> {
    head: Option<NonNull<Node<T>>> 
}

impl <T> LinkedList<T> {
    pub fn new() -> Self {
        return Self {
            head: None 
        };
    }

    pub fn push(&mut self, data: T) {
        let new_node = Box::new(Node::new(data));
        let node_wrapped = NonNull::new(Box::leak(new_node) as *mut Node<T>).unwrap();

        if let Some(head) = self.head {
            unsafe {
                let tail_ptr = (*head.as_ptr()).tail();
                (*tail_ptr.as_ptr()).next = Some(node_wrapped);
            } 
        } else {
            self.head = Some(node_wrapped);
        };
    }
}

impl <T: Debug> LinkedList<T> {
    pub fn debug_print_node_values(&self) {
        let mut current_node: Option<NonNull<Node<T>>> = self.head;

        unsafe {
            let mut node_counter = 0;

            while let Some(node) = current_node {
                let node_ptr = node.as_ptr();
                
                if node_counter > 0 {
                    print!(" -> ");
                }
                print!("{:?}", (*node_ptr).data);

                current_node = (*node_ptr).next;
                node_counter += 1;
            }

            println!();
        }
    }
}

#[derive(Debug)]
struct Node<T> {
    data: T,
    next: Option<NonNull<Node<T>>>
}

impl <T> Node<T> {
    pub fn new(data: T) -> Self {
       return Self {
           data,
           next: None
       }; 
    }

    pub fn tail(&mut self) -> NonNull<Node<T>> {
        let mut current = self as *mut Node<T>;

        unsafe {
            while let Some(node) = (*current).next {
                current = node.as_ptr();
            }
        }
        
        return unsafe { NonNull::new_unchecked(current) };
    }
}

fn main() {
    let mut list: LinkedList<i32> = LinkedList::new();
    list.push(1);
    list.push(2);
    list.push(3);
    list.push(4);
    list.push(5);
    list.push(6);
    list.debug_print_node_values();
}
