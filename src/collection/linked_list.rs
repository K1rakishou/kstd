use crate::{alloc::allocator::Allocator, collection::boxed::KBox};

struct LinkedList<'a, T, A : Allocator> {
    _allocator: &'a A,
    head: Option<KBox<'a, Node<'a, T, A>, A>>,
}

impl<'a, T, A : Allocator> LinkedList<'a, T, A> {
    pub fn new(allocator: &'a A) -> Self {
        return Self {
             _allocator: allocator,
             head: None
         };
    }

    pub fn push(&mut self, data: T) {
        let mut node = KBox::new(self._allocator, Node::new(data));
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

    pub fn reverse(&mut self) {
        let mut prev: Option<KBox<Node<T, A>, A>> = None;
        let mut current = self.head.take();

        while let Some(mut node) = current {
            let next = node.next.take();
            node.next = prev;
            prev = Some(node);
            current = next;
        }

        self.head = prev;
    }
}

struct Node<'a, T, A : Allocator> {
    data: T,
    next: Option<KBox<'a, Node<'a, T, A>, A>>,
}

impl<'a, T, A : Allocator> Node<'a, T, A> {
    pub fn new(data: T) -> Self {
        return Self { data, next: None };
    }
}

mod test {
    use crate::{alloc::{allocator::Allocator, global::GlobalAllocator}, collection::{boxed::KBox, linked_list::{LinkedList, Node}}};
    use std::fmt::Debug;
    
    #[test]
    fn test_linked_list() {
        let allocator = GlobalAllocator::new();
        let mut list: LinkedList<i32, GlobalAllocator> = LinkedList::new(&allocator);
        list.push(1);
        list.push(2);    
        list.push(3);    
        list.push(4);    
        list.push(5);    
        list.push(6);
    
        list.reverse();

        list.pop().unwrap();
        list.pop().unwrap();
        list.pop().unwrap();
        list.pop().unwrap();
        list.pop().unwrap();
        list.pop().unwrap();
    }
}
