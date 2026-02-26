use crate::interpreter::value::Value;
use crate::interpreter::value::Value::Null;

const MAX_HEAP_SIZE: usize = 500_000_000;

#[derive(Debug, Copy, Clone, PartialEq)]
struct AllocInfo {
    ptr: usize,
    len: usize,
}

#[derive(Debug)]
pub struct Heap {
    mem: Vec<Value>,
    ptr_offset: usize,

    allocated: Vec<AllocInfo>
}

impl Heap {
    pub fn new(ptr_offset: usize) -> Self {
        let mem = Vec::with_capacity(1000);

        Self {
            mem, ptr_offset,

            allocated: vec![]
        }
    }

    pub fn alloc(&mut self, size: u32) -> u32 {
        let ptr = self.find_space(size as usize);

        self.allocated.push(AllocInfo{
            ptr,
            len: size as usize
        });

        self.allocated.sort_by_key(|i| i.ptr);

        self.abs_ptr(ptr) as u32
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

    pub fn load(&self, ptr: u32, offset: u32) -> Value{
        let ptr = self.rel_ptr(ptr as usize) + (offset as usize);

        self.mem[ptr]
    }

    pub fn store(&mut self, ptr: u32, offset: u32, value: Value) {
        let ptr = self.rel_ptr(ptr as usize) + (offset as usize);

        if ptr >= self.mem.len() {
            self.mem.resize(ptr+1, Null);
        }

        self.mem[ptr] = value;
    }

    fn find_space(&self, size: usize) -> usize {
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
        let mut heap = Heap::new(20);

        let ptr = heap.alloc(7);

        assert_eq!(ptr, 20);

        assert_eq!(heap.allocated, vec![AllocInfo { ptr: 0, len: 7 }]);
    }

    #[test]
    fn test_alloc() {
        let mut heap = Heap::new(0);

        let value1 = IntValue(5);
        let value2 = IntValue(10);

        let ptr = heap.alloc(5);

        heap.store(ptr, 1, value1);

        let ptr2 = heap.alloc(1);
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
        let mut heap = Heap::new(0);

        heap.alloc(10);

        let mid = heap.alloc(7);

        heap.alloc(10);

        heap.free(mid);

        let ptr = heap.alloc(4);

        assert_eq!(ptr, 10);

        assert_eq!(heap.allocated, vec![AllocInfo { ptr: 0, len: 10 }, AllocInfo { ptr: 10, len: 4 }, AllocInfo { ptr: 17, len: 10 }]);
    }

    #[test]
    fn test_free2() {
        let mut heap = Heap::new(0);

        heap.alloc(10);

        let mid = heap.alloc(7);

        heap.alloc(10);

        heap.free(mid);

        let ptr = heap.alloc(7);

        assert_eq!(ptr, 10);

        assert_eq!(heap.allocated, vec![AllocInfo { ptr: 0, len: 10 }, AllocInfo { ptr: 10, len: 7 }, AllocInfo { ptr: 17, len: 10 }]);
    }

    #[test]
    fn test_free3() {
        let mut heap = Heap::new(0);

        heap.alloc(10);

        let mid = heap.alloc(7);

        heap.alloc(10);

        heap.free(mid);

        let ptr = heap.alloc(8);

        assert_eq!(ptr, 27);

        assert_eq!(heap.allocated, vec![AllocInfo { ptr: 0, len: 10 }, AllocInfo { ptr: 17, len: 10 }, AllocInfo {ptr: 27, len: 8}]);
    }

}