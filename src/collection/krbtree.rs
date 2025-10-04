use std::{cmp::Ordering, hash::Hash, usize};
use std::fmt::Debug;
use crate::{alloc::kallocator::KAllocator, collection::kvec::KVec};

static ENABLE_DEBUG_LOGS: bool = false;

#[derive(Debug, Clone, Copy, PartialEq)]
struct NodeIdx(usize);

#[derive(Debug, PartialEq)]
enum KRBTreeNodeColor {
    Red,
    Black
}

pub struct KRBTree<'a, K : Debug + Hash + Ord, V : Debug + PartialEq, A : KAllocator> {
    _allocator: &'a A,
    _pool: KRBTreeNodePool<'a, K, V, A>,
    _nodes: KVec<'a, NodeIdx, A>
}

#[derive(Debug)]
struct KRBTreeNode<K : Debug + Hash + Ord, V : Debug + PartialEq> {
    _kv: Option<(K, V)>,
    _parent: Option<NodeIdx>,
    _left: Option<NodeIdx>,
    _right: Option<NodeIdx>,
    _color: KRBTreeNodeColor,
    _borrowed: bool
}

struct KRBTreeNodePool<'a, K : Debug + Hash + Ord, V : Debug + PartialEq, A : KAllocator> {
    _nodes: KVec<'a, KRBTreeNode<K, V>, A>
}

impl<'a, K : Debug + Hash + Ord, V : Debug + PartialEq, A : KAllocator> KRBTree<'a, K, V, A> {
    pub fn new(allocator: &'a A, capacity: usize) -> Self {
        return Self {
            _allocator: allocator,
            _pool: KRBTreeNodePool::new(allocator, capacity),
            _nodes: KVec::new(allocator)
        };
    }

    pub fn insert(&mut self, key: K, value: V) {
        let color = KRBTreeNodeColor::Red;
        if ENABLE_DEBUG_LOGS {
            println!("Inserting ({:?}, {:?})", key, value);
        }

        if self._nodes.is_empty() {
            let root_idx = self._pool.borrow(None, KRBTreeNodeColor::Black);
            self._nodes.push(root_idx);
            
            let inserted_node_idx = self._pool.borrow(Some((key, value)), color);
            self._nodes.push(inserted_node_idx);

            self.node_mut(root_idx)._right = Some(inserted_node_idx);
            self.node_mut(inserted_node_idx)._parent = Some(root_idx);

            self.rebalance(inserted_node_idx);
        } else {
            let (inserted_node_idx, need_rebalance) = self.insert_impl(self.first_node_idx(), key, value, color);
            if need_rebalance {
                self.rebalance(inserted_node_idx);
            }
        }
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let mut next_node_idx = self.node(self.root_node_idx())._right.as_ref();

        while let Some(node_idx) = next_node_idx.take() {
            let node = &self.node(*node_idx);
            if ENABLE_DEBUG_LOGS {
                println!("[get] node_idx: {}, node: {:?}", node_idx.0, node._kv);
            }
            
            if let Some(kv) = &node._kv {
                let inserted_key = &kv.0;
                let inserted_value = &kv.1;

                match key.cmp(inserted_key) {
                    Ordering::Less => {
                        next_node_idx = node._left.as_ref();

                        if ENABLE_DEBUG_LOGS {
                            println!("[get] left_node_idx: {:?}", next_node_idx);
                        }
                    },
                    Ordering::Equal => return Some(inserted_value),
                    Ordering::Greater => {
                        next_node_idx = node._right.as_ref();

                        if ENABLE_DEBUG_LOGS {
                            println!("[get] right_node_idx: {:?}", next_node_idx);
                        }
                    },
                }
            }
        }

        return None;
    }

    pub fn length(&self) -> usize {
        if self._nodes.is_empty() {
            return 0;
        }
        
        return self._nodes.len() - 1;
    }

    #[inline]
    fn root_node_idx(&self) -> NodeIdx {
        return *self._nodes.get(0).expect("KRBTree has no root");
    }

    #[inline]
    fn first_node_idx(&self) -> NodeIdx {
        let root_idx = self.root_node_idx();
        return self.node(root_idx)._right.expect("KRBTree is empty");
    }

    #[inline]
    fn node(&self, idx: NodeIdx) -> &KRBTreeNode<K, V> {
        return self._pool.get(idx);
    }

    #[inline]
    fn node_maybe(&self, idx: NodeIdx) -> Option<&KRBTreeNode<K, V>> {
        return self._pool.get_maybe(idx);
    }

    #[inline]
    fn node_mut(&mut self, idx: NodeIdx) -> &mut KRBTreeNode<K, V> {
        return self._pool.get_mut(idx);
    }

    fn insert_impl(&mut self, current_idx: NodeIdx, new_key: K, new_value: V, color: KRBTreeNodeColor) -> (NodeIdx, bool) {
        debug_assert!(!self.is_root(current_idx));

        let current_node = self.node(current_idx);
        let Some(current_node_kv) = &current_node._kv else {
            unreachable!();
        };

        let child_node_idx = match new_key.cmp(&current_node_kv.0) {
            Ordering::Less => {
                match self.node(current_idx)._left {
                    Some(left_idx) => {
                        if ENABLE_DEBUG_LOGS {
                            println!("[insert_impl] going left");
                        }

                        left_idx
                    },
                    None => {
                        if ENABLE_DEBUG_LOGS {
                            println!("[insert_impl] found node at left child, current node: {:?}", current_node._kv);
                        }

                        let new_node_idx = self._pool.borrow(Some((new_key, new_value)), color);

                        self.node_mut(current_idx)._left = Some(new_node_idx);
                        self.node_mut(new_node_idx)._parent = Some(current_idx);
                
                        return (new_node_idx, true);
                    },
                }
            },
            Ordering::Equal => {
                // replace current node's value with the new one
                let current_node = self.node_mut(current_idx);
                current_node._kv.replace((new_key, new_value));

                if ENABLE_DEBUG_LOGS {
                    println!("[insert_impl] keys are equal");
                }
            
                return (current_idx, false);
            },
            Ordering::Greater => {
                match self.node(current_idx)._right {
                    Some(right_idx) => {
                        if ENABLE_DEBUG_LOGS {
                            println!("[insert_impl] going right");
                        }

                        right_idx
                    },
                    None => {
                        if ENABLE_DEBUG_LOGS {
                            println!("[insert_impl] found node at right child, current node: {:?}", current_node._kv);
                        }

                        let new_node_idx = self._pool.borrow(Some((new_key, new_value)), color);
                
                        self.node_mut(current_idx)._right = Some(new_node_idx);
                        self.node_mut(new_node_idx)._parent = Some(current_idx);
                
                        return (new_node_idx, true);
                    },
                }
            },
        };

        return self.insert_impl(child_node_idx, new_key, new_value, color);
    }

    fn rebalance(&mut self, inserted_node_idx: NodeIdx) {
        let mut z_idx = inserted_node_idx;
        
        loop {
            let Some(p_idx) = self.node(z_idx)._parent else {
                break;
            };
            let Some(g_idx) = self.node(p_idx)._parent else {
                break;
            };

            if self.is_root(g_idx) {
                break;
            }

            if self.is_node_black(p_idx) {
                break;
            }

            if ENABLE_DEBUG_LOGS {
                println!("[rebalance], z: {:?}", self.node(z_idx)._kv);
            }

            let g_node = self.node(g_idx);
            let u_idx = if let Some(left_idx) = g_node._left && left_idx != p_idx {
                Some(left_idx)
            } else if let Some(right_idx) = g_node._right && right_idx != p_idx {
                Some(right_idx)
            } else {
                None
            };

            if let Some(u_idx) = u_idx && self.is_node_red(u_idx) {
                if ENABLE_DEBUG_LOGS {
                    println!("u {:?} is red", self.node(u_idx)._kv);
                }
                
                self.color_node_black(p_idx);
                self.color_node_black(u_idx);
                if !self.is_root(g_idx) {
                    self.color_node_red(g_idx);
                }

                z_idx = g_idx;
                continue;
            }

            if self.is_inner_child(z_idx, g_idx) {
                if ENABLE_DEBUG_LOGS {
                    println!("z {:?} is inner child", self.node(z_idx)._kv);
                }
                
                if self.node(p_idx)._right == Some(z_idx) {
                    self.rotate_left(p_idx);
                    z_idx = p_idx;
                } else if self.node(p_idx)._left == Some(z_idx) {
                    self.rotate_right(p_idx);
                    z_idx = p_idx;
                } else {
                    unreachable!();
                }

                continue;
            }

            if self.is_outer_child(z_idx, g_idx) {
                if ENABLE_DEBUG_LOGS {
                    println!("z {:?} is outer child", self.node(z_idx)._kv);
                }
                
                self.color_node_black(p_idx);
                if !self.is_root(g_idx) {
                    self.color_node_red(g_idx);
                }

                if self.node(g_idx)._left == Some(p_idx) {
                    self.rotate_right(g_idx);
                } else if self.node(g_idx)._right == Some(p_idx) {
                    self.rotate_left(g_idx);
                } else {
                    unreachable!();
                }

                break;
            }

            unreachable!();
        }

        let first_node_idx = self.first_node_idx();
        if self.is_node_red(first_node_idx) {
            self.color_node_black(first_node_idx);
        }
    }

    fn rotate_left(&mut self, x_idx: NodeIdx) {
        if ENABLE_DEBUG_LOGS {
            println!("rotate_left around {:?}", self.node(x_idx)._kv);
        }
        
        let Some(y_dx) = self.node(x_idx)._right else {
            unreachable!();
        };

        let b_idx = self.node(y_dx)._left;
        let p_idx = self.node(x_idx)._parent.expect("Parent is None");

        if self.node(p_idx)._left == Some(x_idx) {
            self.node_mut(p_idx)._left = Some(y_dx);
        } else {
            debug_assert_eq!(self.node(p_idx)._right, Some(x_idx));
            self.node_mut(p_idx)._right = Some(y_dx);
        }

        self.node_mut(y_dx)._parent = Some(p_idx);
        self.node_mut(x_idx)._right = b_idx;
        
        if let Some(b_idx) = b_idx {
            self.node_mut(b_idx)._parent = Some(x_idx);
        }

        self.node_mut(y_dx)._left = Some(x_idx);
        self.node_mut(x_idx)._parent = Some(y_dx);
    }

    fn rotate_right(&mut self, x_idx: NodeIdx) {
        if ENABLE_DEBUG_LOGS {
            println!("rotate_right around {:?}", self.node(x_idx)._kv);
        }
        
        let Some(y_idx) = self.node(x_idx)._left else {
            unreachable!();
        };

        let b_idx = self.node(y_idx)._right;
        let p_idx = self.node(x_idx)._parent.expect("Parent is None");

        if self.node(p_idx)._right == Some(x_idx) {
            self.node_mut(p_idx)._right = Some(y_idx);
        } else {
            debug_assert_eq!(self.node(p_idx)._left, Some(x_idx));
            self.node_mut(p_idx)._left = Some(y_idx);
        }
        
        self.node_mut(y_idx)._parent = Some(p_idx);
        self.node_mut(x_idx)._left = b_idx;

        if let Some(b_idx) = b_idx {
            self.node_mut(b_idx)._parent = Some(x_idx);
        }

        self.node_mut(y_idx)._right = Some(x_idx);
        self.node_mut(x_idx)._parent = Some(y_idx);
    }

    #[inline]
    fn is_inner_child(&self, z_idx: NodeIdx, g_idx: NodeIdx) -> bool {
        let g_node = self.node(g_idx);

        if let Some(l_idx) = g_node._left {
            if let Some(lr_idx) = self.node(l_idx)._right {
                if lr_idx == z_idx {
                    return true;
                }
            }
        }
        
        if let Some(r_idx) = g_node._right {
            if let Some(rl_idx) = self.node(r_idx)._left {
                if rl_idx == z_idx {
                    return true;
                }
            }
        }
        
        return false;
    }

    #[inline]
    fn is_outer_child(&self, z_idx: NodeIdx, g_idx: NodeIdx) -> bool {
        let g_node = self.node(g_idx);

        if let Some(l_idx) = g_node._left {
            if let Some(ll_idx) = self.node(l_idx)._left {
                if ll_idx == z_idx {
                    return true;
                }
            }
        }
        
        if let Some(r_idx) = g_node._right {
            if let Some(rr_idx) = self.node(r_idx)._right {
                if rr_idx == z_idx {
                    return true;
                }
            }
        }
        
        return false;
    }

    #[inline]
    fn reset(&mut self, idx: NodeIdx) {
        debug_assert!(!self.is_root(idx), "idx: {}", idx.0);

        self.node_mut(idx).reset();
    }

    #[inline]
    fn is_node_red(&self, idx: NodeIdx) -> bool {
        if self.is_root(idx) {
            return false;
        }
        
        let node = self.node(idx);
        return node._color == KRBTreeNodeColor::Red;
    }

    #[inline]
    fn is_node_black(&self, idx: NodeIdx) -> bool {
        if self.is_root(idx) {
            return true;
        }
        
        let node = self.node(idx);
        return node._color == KRBTreeNodeColor::Black;
    }

    #[inline]
    fn is_root(&self, idx: NodeIdx) -> bool {
        return self.node(idx)._parent.is_none();
    }

    #[inline]
    fn color_node_black(&mut self, idx: NodeIdx) {
        if self.is_node_black(idx) {
            return;
        }
        
        let node = self.node_mut(idx);
        debug_assert!(node._borrowed == true, "node.idx: {}", idx.0);
        node._color = KRBTreeNodeColor::Black;
    }

    #[inline]
    fn color_node_red(&mut self, idx: NodeIdx) {
        if self.is_root(idx) || self.is_node_red(idx) {
            return;
        }
        
        let node = self.node_mut(idx);
        debug_assert!(node._borrowed == true, "node.idx: {}", idx.0);
        node._color = KRBTreeNodeColor::Red;
    }
    
}

impl<'a, K : Debug + Hash + Ord, V : Debug + PartialEq, A : KAllocator> KRBTreeNodePool<'a, K, V, A> {
    fn new(allocator: &'a A, capacity: usize) -> Self {
        return Self {
            _nodes: KVec::with_capacity(allocator, capacity)
        };
    }

    fn borrow(&mut self, kv: Option<(K, V)>, color: KRBTreeNodeColor) -> NodeIdx {
        return match self._nodes.iter_mut().position(|node| !node._borrowed) {
            Some(position) => {
                self._nodes[position]._borrowed = true;
                self._nodes[position]._kv = kv;
                NodeIdx::new(position)
            },
            None => {
                let node_idx = self._nodes.len();
                self._nodes.push(KRBTreeNode::new(kv, color));
                NodeIdx::new(node_idx)
            },
        };
    }

    #[inline]
    fn get_maybe(&self, idx: NodeIdx) -> Option<&KRBTreeNode<K, V>> {
        return self._nodes.get(idx.0);
    }

    #[inline]
    fn get(&self, idx: NodeIdx) -> &KRBTreeNode<K, V> {
        return &self._nodes[idx.0];
    }

    #[inline]
    fn get_mut(&mut self, idx: NodeIdx) -> &mut KRBTreeNode<K, V> {
        return &mut self._nodes[idx.0];
    }
}

impl<K : Debug + Hash + Ord, V : Debug + PartialEq> KRBTreeNode<K, V> {
    fn new(kv: Option<(K, V)>, color: KRBTreeNodeColor) -> Self {
        return Self {
            _kv: kv,
            _parent: None,
            _left: None,
            _right: None,
            _color: color,
            _borrowed: true
        };
    }

    fn reset(&mut self) {
        let _  = self._kv.take();
        self._parent = None;
        self._left = None;
        self._right = None;
        self._color = KRBTreeNodeColor::Black;
        self._borrowed = false;
    }
}

impl NodeIdx {
    fn root() -> Self {
        return Self(0);
    }
    
    fn new(idx: usize) -> Self {
        return Self(idx);
    }
}

#[allow(unused_imports, dead_code)]
mod test {
    use std::process::Command;
    use std::{fmt::Debug, fs::File, hash::Hash};
    use std::io::Write;
    use crate::{alloc::{global::GlobalAllocator, kallocator::KAllocator}, collection::krbtree::{KRBTree, KRBTreeNode, KRBTreeNodeColor, NodeIdx}};

    ///      Before rotation
    ///            11
    ///           /
    ///          5
    ///         /
    ///        2
    /// 
    ///      After rotation
    ///            5
    ///           / \
    ///          2   11 
    /// 
    #[test]
    fn test_krbtree_insert_right_outer_case() {
        let allocator = GlobalAllocator::new();
        let mut krbtree = KRBTree::<usize, usize, GlobalAllocator>::new(&allocator, 4);

        let elements: Vec<usize> = vec![11, 5, 2];

        for e in elements.iter() {
            krbtree.insert(*e, *e);
        }

        let root_idx = NodeIdx::new(0);
        let node_11_idx = NodeIdx::new(1);
        let node_5_idx = NodeIdx::new(2);
        let node_2_idx = NodeIdx::new(3);

        let root = krbtree.node(root_idx);
        assert_eq!(true, root._kv.is_none());
        let node_11 = krbtree.node(node_11_idx);
        assert_eq!(11, node_11._kv.unwrap().0);
        let node_5 = krbtree.node(node_5_idx);
        assert_eq!(5, node_5._kv.unwrap().0);
        let node_2 = krbtree.node(node_2_idx);
        assert_eq!(2, node_2._kv.unwrap().0);

        assert_eq!(true, root._parent.is_none());
        assert_eq!(true, root._left.is_none());
        assert_eq!(node_5_idx, root._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, root._color);

        assert_eq!(root_idx, node_5._parent.unwrap());
        assert_eq!(node_2_idx, node_5._left.unwrap());
        assert_eq!(node_11_idx, node_5._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, node_5._color);
        
        assert_eq!(node_5_idx, node_11._parent.unwrap());
        assert_eq!(true, node_11._left.is_none());
        assert_eq!(true, node_11._right.is_none());
        assert_eq!(KRBTreeNodeColor::Red, node_11._color);
        
        assert_eq!(node_5_idx, node_2._parent.unwrap());
        assert_eq!(true, node_2._left.is_none());
        assert_eq!(true, node_2._right.is_none());
        assert_eq!(KRBTreeNodeColor::Red, node_2._color);
    }
    
    ///      Before rotation
    ///            2
    ///             \
    ///              5
    ///               \
    ///               11
    ///  
    ///      After rotation
    ///            5
    ///           / \
    ///          2   11 
    /// 
    #[test]
    fn test_krbtree_insert_left_rotation_outer_case() {
        let allocator = GlobalAllocator::new();
        let mut krbtree = KRBTree::<usize, usize, GlobalAllocator>::new(&allocator, 4);

        let elements: Vec<usize> = vec![2, 5, 11];

        for e in elements.iter() {
            krbtree.insert(*e, *e);
        }

        let root_idx = NodeIdx::new(0);
        let node_2_idx = NodeIdx::new(1);
        let node_5_idx = NodeIdx::new(2);
        let node_11_idx = NodeIdx::new(3);

        let root = krbtree.node(root_idx);
        assert_eq!(true, root._kv.is_none());
        let node_11 = krbtree.node(node_11_idx);
        assert_eq!(11, node_11._kv.unwrap().0);
        let node_5 = krbtree.node(node_5_idx);
        assert_eq!(5, node_5._kv.unwrap().0);
        let node_2 = krbtree.node(node_2_idx);
        assert_eq!(2, node_2._kv.unwrap().0);

        assert_eq!(true, root._parent.is_none());
        assert_eq!(true, root._left.is_none());
        assert_eq!(node_5_idx, root._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, root._color);

        assert_eq!(root_idx, node_5._parent.unwrap());
        assert_eq!(node_2_idx, node_5._left.unwrap());
        assert_eq!(node_11_idx, node_5._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, node_5._color);
        
        assert_eq!(node_5_idx, node_11._parent.unwrap());
        assert_eq!(true, node_11._left.is_none());
        assert_eq!(true, node_11._right.is_none());
        assert_eq!(KRBTreeNodeColor::Red, node_11._color);
        
        assert_eq!(node_5_idx, node_2._parent.unwrap());
        assert_eq!(true, node_2._left.is_none());
        assert_eq!(true, node_2._right.is_none());
        assert_eq!(KRBTreeNodeColor::Red, node_2._color);
    }

    ///      Before rotation
    ///            11
    ///           /
    ///          5
    ///           \
    ///            7
    /// 
    ///      After rotation
    ///            7
    ///           / \
    ///          5   11 
    /// 
    #[test]
    fn test_krbtree_insert_left_right_rotation_inner_case() {
        let allocator = GlobalAllocator::new();
        let mut krbtree = KRBTree::<usize, usize, GlobalAllocator>::new(&allocator, 4);

        let elements: Vec<usize> = vec![11, 5, 7];

        for e in elements.iter() {
            krbtree.insert(*e, *e);
        }

        let root_idx = NodeIdx::new(0);
        let node_11_idx = NodeIdx::new(1);
        let node_5_idx = NodeIdx::new(2);
        let node_7_idx = NodeIdx::new(3);

        let root = krbtree.node(root_idx);
        assert_eq!(true, root._kv.is_none());
        let node_11 = krbtree.node(node_11_idx);
        assert_eq!(11, node_11._kv.unwrap().0);
        let node_5 = krbtree.node(node_5_idx);
        assert_eq!(5, node_5._kv.unwrap().0);
        let node_7 = krbtree.node(node_7_idx);
        assert_eq!(7, node_7._kv.unwrap().0);

        assert_eq!(true, root._parent.is_none());
        assert_eq!(true, root._left.is_none());
        assert_eq!(node_7_idx, root._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, root._color);

        assert_eq!(root_idx, node_7._parent.unwrap());
        assert_eq!(node_5_idx, node_7._left.unwrap());
        assert_eq!(node_11_idx, node_7._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, node_7._color);
        
        assert_eq!(node_7_idx, node_11._parent.unwrap());
        assert_eq!(true, node_11._left.is_none());
        assert_eq!(true, node_11._right.is_none());
        assert_eq!(KRBTreeNodeColor::Red, node_11._color);
        
        assert_eq!(node_7_idx, node_5._parent.unwrap());
        assert_eq!(true, node_5._left.is_none());
        assert_eq!(true, node_5._right.is_none());
        assert_eq!(KRBTreeNodeColor::Red, node_5._color);
    }

    ///      Before rotation
    ///            11
    ///             \
    ///              17
    ///             /
    ///            13
    /// 
    ///      After rotation
    ///            13
    ///           / \
    ///          11   17 
    /// 
    #[test]
    fn test_krbtree_insert_right_left_rotation_inner_case() {
        let allocator = GlobalAllocator::new();
        let mut krbtree = KRBTree::<usize, usize, GlobalAllocator>::new(&allocator, 4);

        let elements: Vec<usize> = vec![11, 17, 13];

        for e in elements.iter() {
            krbtree.insert(*e, *e);
        }

        let root_idx = NodeIdx::new(0);
        let node_11_idx = NodeIdx::new(1);
        let node_17_idx = NodeIdx::new(2);
        let node_13_idx = NodeIdx::new(3);

        let root = krbtree.node(root_idx);
        assert_eq!(true, root._kv.is_none());
        let node_11 = krbtree.node(node_11_idx);
        assert_eq!(11, node_11._kv.unwrap().0);
        let node_17 = krbtree.node(node_17_idx);
        assert_eq!(17, node_17._kv.unwrap().0);
        let node_13 = krbtree.node(node_13_idx);
        assert_eq!(13, node_13._kv.unwrap().0);

        assert_eq!(true, root._parent.is_none());
        assert_eq!(true, root._left.is_none());
        assert_eq!(node_13_idx, root._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, root._color);

        assert_eq!(root_idx, node_13._parent.unwrap());
        assert_eq!(node_11_idx, node_13._left.unwrap());
        assert_eq!(node_17_idx, node_13._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, node_13._color);
        
        assert_eq!(node_13_idx, node_11._parent.unwrap());
        assert_eq!(true, node_11._left.is_none());
        assert_eq!(true, node_11._right.is_none());
        assert_eq!(KRBTreeNodeColor::Red, node_11._color);
        
        assert_eq!(node_13_idx, node_17._parent.unwrap());
        assert_eq!(true, node_17._left.is_none());
        assert_eq!(true, node_17._right.is_none());
        assert_eq!(KRBTreeNodeColor::Red, node_17._color);
    }

    ///      Before rotation
    ///            100
    ///           /   \
    ///          60   140
    ///         /
    ///        50
    ///         \
    ///         55
    /// 
    ///      After rotation
    ///            100
    ///           /   \
    ///          55   140
    ///         /  \
    ///        50  60
    #[test]
    fn test_krbtree_insert_left_right_under_left_child_of_parent() {
        let allocator = GlobalAllocator::new();
        let mut krbtree = KRBTree::<usize, usize, GlobalAllocator>::new(&allocator, 4);

        let elements: Vec<usize> = vec![100, 60, 140, 50, 55];

        for e in elements.iter() {
            krbtree.insert(*e, *e);
        }

        let root_idx = NodeIdx::new(0);
        let node_100_idx = NodeIdx::new(1);
        let node_60_idx = NodeIdx::new(2);
        let node_140_idx = NodeIdx::new(3);
        let node_50_idx = NodeIdx::new(4);
        let node_55_idx = NodeIdx::new(5);

        let root = krbtree.node(root_idx);
        assert_eq!(true, root._kv.is_none());
        let node_100 = krbtree.node(node_100_idx);
        assert_eq!(100, node_100._kv.unwrap().0);
        let node_60 = krbtree.node(node_60_idx);
        assert_eq!(60, node_60._kv.unwrap().0);
        let node_140 = krbtree.node(node_140_idx);
        assert_eq!(140, node_140._kv.unwrap().0);
        let node_50 = krbtree.node(node_50_idx);
        assert_eq!(50, node_50._kv.unwrap().0);
        let node_55 = krbtree.node(node_55_idx);
        assert_eq!(55, node_55._kv.unwrap().0);

        assert_eq!(true, root._parent.is_none());
        assert_eq!(true, root._left.is_none());
        assert_eq!(node_100_idx, root._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, root._color);

        assert_eq!(root_idx, node_100._parent.unwrap());
        assert_eq!(node_55_idx, node_100._left.unwrap());
        assert_eq!(node_140_idx, node_100._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, node_100._color);

        assert_eq!(node_100_idx, node_55._parent.unwrap());
        assert_eq!(node_50_idx, node_55._left.unwrap());
        assert_eq!(node_60_idx, node_55._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, node_55._color);

        assert_eq!(node_100_idx, node_140._parent.unwrap());
        assert_eq!(true, node_140._left.is_none());
        assert_eq!(true, node_140._right.is_none());
        assert_eq!(KRBTreeNodeColor::Black, node_140._color);

        assert_eq!(node_55_idx, node_50._parent.unwrap());
        assert_eq!(true, node_50._left.is_none());
        assert_eq!(true, node_50._right.is_none());
        assert_eq!(KRBTreeNodeColor::Red, node_50._color);

        assert_eq!(node_55_idx, node_60._parent.unwrap());
        assert_eq!(true, node_60._left.is_none());
        assert_eq!(true, node_60._right.is_none());
        assert_eq!(KRBTreeNodeColor::Red, node_60._color);
    }

    ///      Before rotation
    ///            100
    ///           /   \
    ///          60   140
    ///                 \
    ///                150
    ///                 /
    ///               145
    /// 
    ///      After rotation
    ///            100
    ///           /   \
    ///          60   145
    ///               / \
    ///             140 150   
    #[test]
    fn test_krbtree_insert_right_left_under_right_child_of_parent() {
        let allocator = GlobalAllocator::new();
        let mut krbtree = KRBTree::<usize, usize, GlobalAllocator>::new(&allocator, 4);

        let elements: Vec<usize> = vec![100, 60, 140, 150, 145];

        for e in elements.iter() {
            krbtree.insert(*e, *e);
        }

        let root_idx = NodeIdx::new(0);
        let node_100_idx = NodeIdx::new(1);
        let node_60_idx = NodeIdx::new(2);
        let node_140_idx = NodeIdx::new(3);
        let node_150_idx = NodeIdx::new(4);
        let node_145_idx = NodeIdx::new(5);

        let root = krbtree.node(root_idx);
        assert_eq!(true, root._kv.is_none());
        let node_100 = krbtree.node(node_100_idx);
        assert_eq!(100, node_100._kv.unwrap().0);
        let node_60 = krbtree.node(node_60_idx);
        assert_eq!(60, node_60._kv.unwrap().0);
        let node_140 = krbtree.node(node_140_idx);
        assert_eq!(140, node_140._kv.unwrap().0);
        let node_150 = krbtree.node(node_150_idx);
        assert_eq!(150, node_150._kv.unwrap().0);
        let node_145 = krbtree.node(node_145_idx);
        assert_eq!(145, node_145._kv.unwrap().0);

        assert_eq!(true, root._parent.is_none());
        assert_eq!(true, root._left.is_none());
        assert_eq!(node_100_idx, root._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, root._color);

        assert_eq!(root_idx, node_100._parent.unwrap());
        assert_eq!(node_60_idx, node_100._left.unwrap());
        assert_eq!(node_145_idx, node_100._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, node_100._color);

        assert_eq!(node_100_idx, node_60._parent.unwrap());
        assert_eq!(true, node_60._left.is_none());
        assert_eq!(true, node_60._right.is_none());
        assert_eq!(KRBTreeNodeColor::Black, node_60._color);

        assert_eq!(node_100_idx, node_145._parent.unwrap());
        assert_eq!(node_140_idx, node_145._left.unwrap());
        assert_eq!(node_150_idx, node_145._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, node_145._color);

        assert_eq!(node_145_idx, node_140._parent.unwrap());
        assert_eq!(true, node_140._left.is_none());
        assert_eq!(true, node_140._right.is_none());
        assert_eq!(KRBTreeNodeColor::Red, node_140._color);

        assert_eq!(node_145_idx, node_150._parent.unwrap());
        assert_eq!(true, node_150._left.is_none());
        assert_eq!(true, node_150._right.is_none());
        assert_eq!(KRBTreeNodeColor::Red, node_150._color);
    }

    #[test]
    fn test_krbtree_insert_1_to_8() {
        let allocator = GlobalAllocator::new();
        let mut krbtree = KRBTree::<usize, usize, GlobalAllocator>::new(&allocator, 4);

        let elements: Vec<usize> = vec![1, 2, 3, 4, 5, 6, 7, 8];

        for e in elements.iter() {
            krbtree.insert(*e, *e);
        }

        let root_idx = NodeIdx::new(0);
        let node_1_idx = NodeIdx::new(1);
        let node_2_idx = NodeIdx::new(2);
        let node_3_idx = NodeIdx::new(3);
        let node_4_idx = NodeIdx::new(4);
        let node_5_idx = NodeIdx::new(5);
        let node_6_idx = NodeIdx::new(6);
        let node_7_idx = NodeIdx::new(7);
        let node_8_idx = NodeIdx::new(8);

        let root = krbtree.node(root_idx);
        assert_eq!(true, root._kv.is_none());
        let node_1 = krbtree.node(node_1_idx);
        assert_eq!(1, node_1._kv.unwrap().0);
        let node_2 = krbtree.node(node_2_idx);
        assert_eq!(2, node_2._kv.unwrap().0);
        let node_3 = krbtree.node(node_3_idx);
        assert_eq!(3, node_3._kv.unwrap().0);
        let node_4 = krbtree.node(node_4_idx);
        assert_eq!(4, node_4._kv.unwrap().0);
        let node_5 = krbtree.node(node_5_idx);
        assert_eq!(5, node_5._kv.unwrap().0);
        let node_6 = krbtree.node(node_6_idx);
        assert_eq!(6, node_6._kv.unwrap().0);
        let node_7 = krbtree.node(node_7_idx);
        assert_eq!(7, node_7._kv.unwrap().0);
        let node_8 = krbtree.node(node_8_idx);
        assert_eq!(8, node_8._kv.unwrap().0);

        assert_eq!(true, root._parent.is_none());
        assert_eq!(true, root._left.is_none());
        assert_eq!(node_4_idx, root._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, root._color);

        assert_eq!(root_idx, node_4._parent.unwrap());
        assert_eq!(node_2_idx, node_4._left.unwrap());
        assert_eq!(node_6_idx, node_4._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, node_4._color);

        // Left sub-tree
        {
            assert_eq!(node_4_idx, node_2._parent.unwrap());
            assert_eq!(node_1_idx, node_2._left.unwrap());
            assert_eq!(node_3_idx, node_2._right.unwrap());
            assert_eq!(KRBTreeNodeColor::Red, node_2._color);

            assert_eq!(node_2_idx, node_1._parent.unwrap());
            assert_eq!(true, node_1._left.is_none());
            assert_eq!(true, node_1._right.is_none());
            assert_eq!(KRBTreeNodeColor::Black, node_1._color);

            assert_eq!(node_2_idx, node_3._parent.unwrap());
            assert_eq!(true, node_3._left.is_none());
            assert_eq!(true, node_3._right.is_none());
            assert_eq!(KRBTreeNodeColor::Black, node_3._color);
        }

        // Right-sub-tree
        {
            
            assert_eq!(node_4_idx, node_6._parent.unwrap());
            assert_eq!(node_5_idx, node_6._left.unwrap());
            assert_eq!(node_7_idx, node_6._right.unwrap());
            assert_eq!(KRBTreeNodeColor::Red, node_6._color);

            assert_eq!(node_6_idx, node_5._parent.unwrap());
            assert_eq!(true, node_5._left.is_none());
            assert_eq!(true, node_5._right.is_none());
            assert_eq!(KRBTreeNodeColor::Black, node_5._color);

            assert_eq!(node_6_idx, node_7._parent.unwrap());
            assert_eq!(true, node_7._left.is_none());
            assert_eq!(node_8_idx, node_7._right.unwrap());
            assert_eq!(KRBTreeNodeColor::Black, node_7._color);

            assert_eq!(node_7_idx, node_8._parent.unwrap());
            assert_eq!(true, node_8._left.is_none());
            assert_eq!(true, node_8._right.is_none());
            assert_eq!(KRBTreeNodeColor::Red, node_8._color);
        }
    }

    #[test]
    fn test_krbtree_insert_8_to_1() {
        let allocator = GlobalAllocator::new();
        let mut krbtree = KRBTree::<usize, usize, GlobalAllocator>::new(&allocator, 4);

        let elements: Vec<usize> = vec![8, 7, 6, 5, 4, 3, 2, 1];

        for e in elements.iter() {
            krbtree.insert(*e, *e);
        }

        let root_idx = NodeIdx::new(0);
        let node_1_idx = NodeIdx::new(8);
        let node_2_idx = NodeIdx::new(7);
        let node_3_idx = NodeIdx::new(6);
        let node_4_idx = NodeIdx::new(5);
        let node_5_idx = NodeIdx::new(4);
        let node_6_idx = NodeIdx::new(3);
        let node_7_idx = NodeIdx::new(2);
        let node_8_idx = NodeIdx::new(1);

        let root = krbtree.node(root_idx);
        assert_eq!(true, root._kv.is_none());
        let node_1 = krbtree.node(node_1_idx);
        assert_eq!(1, node_1._kv.unwrap().0);
        let node_2 = krbtree.node(node_2_idx);
        assert_eq!(2, node_2._kv.unwrap().0);
        let node_3 = krbtree.node(node_3_idx);
        assert_eq!(3, node_3._kv.unwrap().0);
        let node_4 = krbtree.node(node_4_idx);
        assert_eq!(4, node_4._kv.unwrap().0);
        let node_5 = krbtree.node(node_5_idx);
        assert_eq!(5, node_5._kv.unwrap().0);
        let node_6 = krbtree.node(node_6_idx);
        assert_eq!(6, node_6._kv.unwrap().0);
        let node_7 = krbtree.node(node_7_idx);
        assert_eq!(7, node_7._kv.unwrap().0);
        let node_8 = krbtree.node(node_8_idx);
        assert_eq!(8, node_8._kv.unwrap().0);

        assert_eq!(true, root._parent.is_none());
        assert_eq!(true, root._left.is_none());
        assert_eq!(node_5_idx, root._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, root._color);

        assert_eq!(root_idx, node_5._parent.unwrap());
        assert_eq!(node_3_idx, node_5._left.unwrap());
        assert_eq!(node_7_idx, node_5._right.unwrap());
        assert_eq!(KRBTreeNodeColor::Black, node_5._color);

        // Left sub-tree
        {
            assert_eq!(node_5_idx, node_3._parent.unwrap());
            assert_eq!(node_2_idx, node_3._left.unwrap());
            assert_eq!(node_4_idx, node_3._right.unwrap());
            assert_eq!(KRBTreeNodeColor::Red, node_3._color);

            assert_eq!(node_3_idx, node_2._parent.unwrap());
            assert_eq!(node_1_idx, node_2._left.unwrap());
            assert_eq!(true, node_2._right.is_none());
            assert_eq!(KRBTreeNodeColor::Black, node_2._color);

            assert_eq!(node_3_idx, node_4._parent.unwrap());
            assert_eq!(true, node_4._left.is_none());
            assert_eq!(true, node_4._right.is_none());
            assert_eq!(KRBTreeNodeColor::Black, node_4._color);
        }

        // Right-sub-tree
        {
            
            assert_eq!(node_5_idx, node_7._parent.unwrap());
            assert_eq!(node_6_idx, node_7._left.unwrap());
            assert_eq!(node_8_idx, node_7._right.unwrap());
            assert_eq!(KRBTreeNodeColor::Red, node_7._color);

            assert_eq!(node_7_idx, node_6._parent.unwrap());
            assert_eq!(true, node_6._left.is_none());
            assert_eq!(true, node_6._right.is_none());
            assert_eq!(KRBTreeNodeColor::Black, node_6._color);

            assert_eq!(node_7_idx, node_8._parent.unwrap());
            assert_eq!(true, node_8._left.is_none());
            assert_eq!(true, node_8._right.is_none());
            assert_eq!(KRBTreeNodeColor::Black, node_8._color);

            assert_eq!(node_7_idx, node_8._parent.unwrap());
            assert_eq!(true, node_8._left.is_none());
            assert_eq!(true, node_8._right.is_none());
            assert_eq!(KRBTreeNodeColor::Black, node_8._color);
        }
    }

    #[test]
    fn test_krbtree_insert_get() {
        let allocator = GlobalAllocator::new();
        let mut krbtree = KRBTree::<usize, usize, GlobalAllocator>::new(&allocator, 4);
        let elements_count = 1024;

        for e in 0..elements_count {
            krbtree.insert(e, e);
        }

        for e in 0..elements_count {
            assert_eq!(e, *krbtree.get(e).unwrap());
        }
    }

    impl<'a, K : Hash + Ord + Debug, V : PartialEq + Debug, A : KAllocator> KRBTree<'a, K, V, A> {
        fn dump_into_graphviz_dot_file(&self, filename: &str) {
            println!("dump_into_graphviz_dot_file");
            
            let mut graph_out = String::new();
            let mut nil_counter = 0;
            graph_out.push_str("digraph RBTree {\n");
            graph_out.push_str("node [shape=circle, style=filled, fontcolor=white];\n");
            
            self.iterate_first(&mut graph_out, vec![self._nodes[0]], &mut |krbtree, out, node_idx, node| {
                match node_idx {
                    Some(node_idx) => {
                        let node_id = node_idx.0;
                        let label = if krbtree.is_root(node_idx) {
                            "(Root)".to_owned()
                        } else {
                            format!("({:?})", node._kv)
                        };
                        let fillcolor = match node._color {
                            KRBTreeNodeColor::Red => "#8b0000",
                            KRBTreeNodeColor::Black => "#333333",
                        };
                        out.push_str(&format!("n{} [label=\"{}\", fillcolor=\"{}\"];\n", node_id, label, fillcolor));
                    },
                    None => {
                        out.push_str(&format!("nil{} [shape=point];\n", nil_counter));
                        nil_counter += 1;
                    },
                }
            });

            nil_counter = 0;

            self.iterate_second(&mut graph_out, self._nodes[0], &mut |krbtree, out, parent_idx, node_idx| {
                match node_idx {
                    Some(node_idx) => {
                        out.push_str(&format!("n{} -> n{}\n", parent_idx.0, node_idx.0));

                        if let Some(parent) = krbtree.node(node_idx)._parent {
                            out.push_str(&format!("n{} -> n{}\n", node_idx.0, parent.0));
                        }
                    },
                    None => {
                        out.push_str(&format!("n{} -> nil{}\n", parent_idx.0, nil_counter));
                        nil_counter += 1;
                    },
                }
            });

            graph_out.push_str("}\n");

            let mut file = File::create(format!("{}.dot", filename)).unwrap();
            write!(file, "{}", graph_out).unwrap();

            Command::new("dot")
               .arg("-Tpng")
               .arg(format!("{}.dot", filename))
               .arg("-o")
               .arg(format!("{}.png", filename))
               .spawn()
               .expect("failed to execute command");
        }

        fn iterate_first<F>(&self, graph_out: &mut String, node_indices: Vec<NodeIdx>, f: &mut F)
        where
            F : FnMut(&KRBTree<'a, K, V, A>, &mut String, Option<NodeIdx>, &KRBTreeNode<K, V>) -> ()
        {
            let mut node_sibling_indexes = Vec::new();
            
            for node_index in node_indices.iter() {
                let node = self.node(*node_index);
                f(self, graph_out, Some(*node_index), node);

                if let Some(left) = node._left {
                    node_sibling_indexes.push(left);
                } else {
                    f(self, graph_out,None, node);
                }

                if let Some(right) = node._right {
                    node_sibling_indexes.push(right);
                } else {
                    f(self, graph_out,None, node);
                }
            }

            if node_sibling_indexes.len() > 0 {
                self.iterate_first(graph_out, node_sibling_indexes, f);
            }
        }

        fn iterate_second<F>(&self, graph_out: &mut String, node_index: NodeIdx, f: &mut F)
        where
            F : FnMut(&KRBTree<'a, K, V, A>, &mut String, NodeIdx, Option<NodeIdx>) -> ()
        {
            let node = self.node(node_index);

            f(self, graph_out, node_index, node._left);
            f(self, graph_out, node_index, node._right);

            if let Some(left) = node._left {
                self.iterate_second(graph_out, left, f);
            }
            if let Some(right) = node._right {
                self.iterate_second(graph_out, right, f);
            }
        }
    }    
}
