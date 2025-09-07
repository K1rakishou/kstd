use std::{cmp::Ordering, ops::ControlFlow};

use crate::{alloc::allocator::Allocator, collection::vec::KVec};

#[derive(Clone)]
pub enum HeapType {
    Min,
    Max
}

pub struct Heap<'a, T : PartialOrd, A : Allocator> {
    _allocator: &'a A,
    _elements: KVec<'a, T, A>,
    _type: HeapType
}

impl<'a, T : PartialOrd, A : Allocator> Heap<'a, T, A> {
    pub fn min(allocator: &'a A) -> Self {
        return Self::new(allocator, HeapType::Min);
    }

    pub fn max(allocator: &'a A) -> Self {
        return Self::new(allocator, HeapType::Max);
    }

    pub fn new(allocator: &'a A, ty: HeapType) -> Self {
        let elements = KVec::<T, A>::new(allocator);
        
        return Self {
            _allocator: allocator,
            _elements: elements,
            _type: ty
        };
    }

    pub fn push(&mut self, value: T) {
        let mut index = self._elements.len();
        self._elements.push(value);

        if index == 0 {
            // Root initialization, no need to do anything else
            return;
        }

        loop {
            let Some(current_element) = self._elements.get(index) else {
                break;
            };

            let Some(parent_index) = self.parent_index(index) else {
                break;
            };

            let Some(parent_element) = self._elements.get(parent_index) else {
                panic!("No element at index {}", parent_index);
            };
            
            match self._type {
                HeapType::Min => {
                    if parent_element <= current_element  {
                        break
                    }
                }
                HeapType::Max => {
                    if parent_element >= current_element  {
                        break
                    }
                }
            }

            self._elements.swap(parent_index, index);
            index = parent_index;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let Some(last_element_index) = self._elements.last_index() else {
            return None;
        };

        if last_element_index == 0 {
            return self._elements.pop();
        }

        self._elements.swap(0, last_element_index);

        let Some(return_element) = self._elements.pop() else {
            panic!("Elements are empty");
        };
        let mut anchor_index = 0;

        loop {
            let Some(current_element) = self._elements.get(anchor_index) else {
                break;
            };
            
            let swap_index = self.find_swap_index(current_element, anchor_index, |a, b| {
                match self._type {
                    HeapType::Min => {
                        return a.partial_cmp(b).expect("Comparison failed");
                    }
                    HeapType::Max => {
                        return b.partial_cmp(a).expect("Comparison failed");
                    }
                }
            });

            let Some(swap_index) = swap_index else {
                break
            };

            self._elements.swap(swap_index, anchor_index);
            anchor_index = swap_index;
        }

        return Some(return_element);
    }

    fn find_swap_index<F : Fn(&T, &T) -> Ordering>(
        &self,
        current_element: &T,
        anchor_index: usize,
        cmp_func: F
    ) -> Option<usize> {
        let mut swap_index: Option<usize> = None;

        let left_index = self.left_child_index(anchor_index);
        let left_element = self._elements.get(left_index);

        let right_index = self.right_child_index(anchor_index);
        let right_element = self._elements.get(right_index);
        
        match (left_element, right_element) {
            (Some(left_element), Some(right_element)) => {
                if cmp_func(left_element, right_element).is_le() && cmp_func(current_element, left_element).is_gt() {
                    swap_index = Some(left_index);
                } else if cmp_func(right_element, left_element).is_le() && cmp_func(current_element, right_element).is_gt() {
                    swap_index = Some(right_index);
                }
            }
            (Some(left_element), None) => {
                if cmp_func(current_element, left_element).is_gt() {
                    swap_index = Some(left_index);
                }
            }
            (None, Some(right_element)) => {
                if cmp_func(current_element, right_element).is_gt() {
                    swap_index = Some(right_index);
                }
            }
            _ => {
                // no-op
            }
        }

        return swap_index;
    }

    fn left_child_index(&self, anchor: usize) -> usize {
        return (2 * anchor) + 1;
    }

    fn right_child_index(&self, anchor: usize) -> usize {
        return (2 * anchor) + 2;
    }

    fn parent_index(&self, anchor: usize) -> Option<usize> {
        let anchor = anchor as f32;

        let parent_index = (anchor - 1.0) / 2.0;
        if parent_index < 0.0 {
            return None;
        }
        
        return Some(f32::floor(parent_index) as usize);
    }
}

mod tests {
    use crate::alloc::{allocator::Allocator, global::GlobalAllocator};
    use super::Heap;
    use std::fmt::Debug;

    #[test]
    fn test_min_heap() {
        let allocator = GlobalAllocator::new();
        let mut heap = Heap::<usize, GlobalAllocator>::min(&allocator);
        heap.push(9);
        heap.push(7);
        heap.push(8);
        heap.push(3);
        heap.push(5);
        heap.push(5);
        heap.push(1);
        heap.push(3);
        heap.push(1);

        assert_eq!(1, heap.pop().unwrap());
        assert_eq!(1, heap.pop().unwrap());
        assert_eq!(3, heap.pop().unwrap());
        assert_eq!(3, heap.pop().unwrap());
        assert_eq!(5, heap.pop().unwrap());
        assert_eq!(5, heap.pop().unwrap());
        assert_eq!(7, heap.pop().unwrap());
        assert_eq!(8, heap.pop().unwrap());
        assert_eq!(9, heap.pop().unwrap());
        assert!(heap.pop().is_none());
    }

    #[test]
    fn test_max_heap() {
        let allocator = GlobalAllocator::new();
        let mut heap = Heap::<usize, GlobalAllocator>::max(&allocator);
        heap.push(9);
        heap.push(7);
        heap.push(8);
        heap.push(3);
        heap.push(5);
        heap.push(5);
        heap.push(1);
        heap.push(3);
        heap.push(1);

        assert_eq!(9, heap.pop().unwrap());
        assert_eq!(8, heap.pop().unwrap());
        assert_eq!(7, heap.pop().unwrap());
        assert_eq!(5, heap.pop().unwrap());
        assert_eq!(5, heap.pop().unwrap());
        assert_eq!(3, heap.pop().unwrap());
        assert_eq!(3, heap.pop().unwrap());
        assert_eq!(1, heap.pop().unwrap());
        assert_eq!(1, heap.pop().unwrap());
        assert!(heap.pop().is_none());
    }

    impl<'a, T : PartialOrd + Debug, A : Allocator> Heap<'a, T, A> {
        fn print_heap(&self) {
            for (index, element) in self._elements.iter().enumerate() {
                if index > 0 {
                    print!(", ");
                }
                
                print!("{:?}", element);
            }

            println!();
        }
    }
}
