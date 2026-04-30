use std::collections::HashSet;
use crate::error::runtime_error::RuntimeError;
use crate::error::runtime_error::RuntimeError::OutOfMemory;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::value::Value;
use crate::interpreter::value::Value::Null;


#[derive(Debug, Copy, Clone, PartialEq)]
struct AllocInfo {
    ptr: usize,
    len: usize,
}

#[derive(Debug)]
pub struct Heap {
    heap_size: usize,

    mem: Vec<Value>,
    ptr_offset: usize,

    free_space: usize,

    allocated: Vec<AllocInfo>
}

impl Heap {
    pub fn new(ptr_offset: usize, heap_size: usize) -> Self {
        let mem = Vec::with_capacity(1000);

        Self {
            heap_size, mem, ptr_offset,

            free_space: heap_size,

            allocated: vec![]
        }
    }

    pub fn alloc(&mut self, size: u32) -> Result<u32, RuntimeError> {
        self._alloc(size, None)
    }

    pub fn gc_alloc(&mut self, size: u32, stack: &[Value]) -> Result<u32, RuntimeError> {
        self._alloc(size, Some(stack))
    }

    pub fn _alloc(&mut self, size: u32, stack: Option<&[Value]>) -> Result<u32, RuntimeError> {
        let ptr = self.find_space(size as usize, stack);

        self.allocated.push(AllocInfo{
            ptr,
            len: size as usize
        });
        self.free_space -= size as usize;

        if ptr + (size as usize) >= self.heap_size {
            return Err(OutOfMemory);
        }

        self.allocated.sort_by_key(|i| i.ptr);

        Ok(self.abs_ptr(ptr) as u32)
    }

    pub fn free(&mut self, ptr: u32) {
        let ptr = self.rel_ptr(ptr as usize);

        let prev_len = self.allocated.len();

        self.allocated.retain(|i| i.ptr != ptr);

        let after_len = self.allocated.len();

        if after_len +1 != prev_len {
            panic!("Invalid free!")
        }
    }

    /// tries to collect unused objects in the heap
    pub fn gc(&mut self, stack: &[Value]) {
        let mut used = HashSet::new();

        for i in 0..stack.len() {
            self.mark(i, stack, &mut used);
        }

        println!("marked {used:?}");

        self.allocated.retain(|a| {
            if used.contains(&(a.ptr as u32)) {
                true
            } else {
                self.free_space += a.len;

                false
            }
        });
    }

    fn mark(&mut self, mem_ptr: usize, stack: &[Value], used: &mut HashSet<u32>) {
        let val;
        if mem_ptr < used.len() {
            val = stack[mem_ptr];
        } else {
            if mem_ptr < self.ptr_offset {
                return;
            }

            let rel_ptr = self.rel_ptr(mem_ptr);
            if rel_ptr >= self.mem.len() {
                return;
            }
            val = self.mem[rel_ptr];
        }

        let (ptr, len) = if let Value::RefValue {ptr, len} = val {
            (ptr, len)
        } else {
            return;
        };
        if used.contains(&ptr) {
            return;
        }

        used.insert(ptr);

        for i in ptr..(ptr+len) {
            self.mark(i as usize, stack, used);
        }
    }

    pub fn load(&self, ptr: u32, offset: u32) -> Value{
        let ptr = self.rel_ptr(ptr as usize) + (offset as usize);

        if ptr >= self.mem.len() {
            return Null;
        }

        self.mem[ptr]
    }

    pub fn store(&mut self, ptr: u32, offset: u32, value: Value) {
        let ptr = self.rel_ptr(ptr as usize) + (offset as usize);

        if ptr >= self.mem.len() {
            self.mem.resize(ptr+1, Null);
        }

        self.mem[ptr] = value;
    }


    fn find_space(&mut self, size: usize, stack: Option<&[Value]>) -> usize {
        println!("find space");
        self._find_space(size, stack, true)
    }

    fn _find_space(&mut self, size: usize, stack: Option<&[Value]>, try_free: bool) -> usize {
        let mut iter = self.allocated.iter();

        let prev = iter.next();

        if prev.is_none() {
            return 0;
        }
        let mut prev = prev.unwrap();

        for info in iter {
            let prev_end_ptr = prev.ptr + prev.len;

            let free = info.ptr - prev_end_ptr;

            if free >= size {
                return prev_end_ptr;
            }

            prev = info;
        }

        if (prev.ptr + prev.len + size) >= self.heap_size && try_free && let Some(s) = stack {
            self.gc(s);
            return self._find_space(size, stack, false);
        }
        if (prev.ptr + prev.len + size) >= self.heap_size {
            println!("OOM {try_free}, {stack:?}");
        }

        prev.ptr + prev.len
    }



    fn abs_ptr(&self, ptr: usize) -> usize {
        self.ptr_offset + ptr
    }

    fn rel_ptr(&self, ptr: usize) -> usize {
        ptr - self.ptr_offset
    }

}


mod test {
    use crate::interpreter::heap::{AllocInfo, Heap};
    use crate::interpreter::value::Value::IntValue;

    #[test]
    fn test_offset() {
        let mut heap = Heap::new(20, 100);

        let ptr = heap.alloc(7).unwrap();

        assert_eq!(ptr, 20);

        assert_eq!(heap.allocated, vec![AllocInfo { ptr: 0, len: 7 }]);
    }

    #[test]
    fn test_alloc() {
        let mut heap = Heap::new(0, 100);

        let value1 = IntValue(5);
        let value2 = IntValue(10);

        let ptr = heap.alloc(5).unwrap();

        heap.store(ptr, 1, value1);

        let ptr2 = heap.alloc(1).unwrap();
        heap.store(ptr2, 0, value2);

        assert!(heap.mem.len() > 5);

        assert_eq!(heap.allocated, vec![AllocInfo { ptr: 0, len: 5 }, AllocInfo { ptr: 5, len: 1 }]);

        assert_eq!(heap.mem[1], value1);
        assert_eq!(heap.mem[5], value2);

        assert_eq!(heap.load(ptr, 1), value1);
        assert_eq!(heap.load(ptr2, 0), value2);
    }

    #[test]
    fn test_free() {
        let mut heap = Heap::new(0, 100);

        heap.alloc(10).unwrap();

        let mid = heap.alloc(7).unwrap();

        heap.alloc(10).unwrap();

        heap.free(mid);

        let ptr = heap.alloc(4).unwrap();

        assert_eq!(ptr, 10);

        assert_eq!(heap.allocated, vec![AllocInfo { ptr: 0, len: 10 }, AllocInfo { ptr: 10, len: 4 }, AllocInfo { ptr: 17, len: 10 }]);
    }

    #[test]
    fn test_free2() {
        let mut heap = Heap::new(0, 100);

        heap.alloc(10).unwrap();

        let mid = heap.alloc(7).unwrap();

        heap.alloc(10).unwrap();

        heap.free(mid);

        let ptr = heap.alloc(7).unwrap();

        assert_eq!(ptr, 10);

        assert_eq!(heap.allocated, vec![AllocInfo { ptr: 0, len: 10 }, AllocInfo { ptr: 10, len: 7 }, AllocInfo { ptr: 17, len: 10 }]);
    }

    #[test]
    fn test_free3() {
        let mut heap = Heap::new(0, 100);

        heap.alloc(10).unwrap();

        let mid = heap.alloc(7).unwrap();

        heap.alloc(10).unwrap();

        heap.free(mid);

        let ptr = heap.alloc(8).unwrap();

        assert_eq!(ptr, 27);

        assert_eq!(heap.allocated, vec![AllocInfo { ptr: 0, len: 10 }, AllocInfo { ptr: 17, len: 10 }, AllocInfo {ptr: 27, len: 8}]);
    }

}