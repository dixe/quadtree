use std::fmt;
use std::ops::{Index, IndexMut};


#[derive(Debug)]
pub struct FreeItem<T> {
    pub item: T,
    next: i32
}

pub struct FreeList<T> {
    data: Vec::<FreeItem<T>>,
    first_free: i32,
    elements : i32
}


impl<T : > FreeList<T> {

    pub fn new() -> Self {
        FreeList {
            data: Vec::new(),
            first_free: -1,
            elements: 0
        }
    }

    pub fn insert(&mut self, item: T) -> i32 {
        self.elements += 1;
        if self.first_free != -1 {
            let index = self.first_free;
            self.first_free = self.data[self.first_free as usize].next;
            self.data[index as usize].item = item;
            self.data[index as usize].next = -1;
            return index;
        }
        else {
            let fe = FreeItem {
                item,
                next: -1
            };

            self.data.push(fe);
            return (self.data.len() - 1) as i32;
        }
    }

    pub fn elements_count(&self) -> i32 {
        self.elements
    }

    pub fn data_len(&self) -> i32 {
        self.data.len() as i32
    }

    pub fn erase(&mut self, n: i32) {
        self.elements -= 1;
        self.data[n as usize].next = self.first_free;
        self.first_free = n;
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.first_free = -1;
        self.elements = 0;
    }


}


impl<T> Index<i32> for FreeList<T> {
    type Output = T;

    fn index<'a>(&'a self, i: i32) -> &'a T {
        &self.data[i as usize].item
    }
}


impl<T> IndexMut<i32> for FreeList<T> {
    fn index_mut<'a>(&'a mut self, i: i32) -> &'a mut T {
        &mut self.data[i as usize].item
    }
}


impl<T: std::fmt::Debug> fmt::Debug for FreeList<T> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg_list = f.debug_list();

         for (i, e) in self.data.iter().enumerate() {
            if e.next == -1 && self.first_free != i as i32 {
                dbg_list.entry(&e.item);
            }
         }
        dbg_list.finish()

    }

}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn t1() {
        let mut fl = FreeList::new();

        let idx1 = fl.insert(3);
        let idx2 = fl.insert(4);

        fl.erase(idx1);
        assert_eq!(fl[idx2], 4);

        let idx3 = fl.insert(2);
        assert_eq!(idx1, idx3); // freeing idx2 is reused as idx3
        assert_eq!(fl[idx3], 2); // new item is retrived

        fl.erase(idx2);
        fl.erase(idx3);

        let idx = fl.insert(1);
        assert_eq!(idx, 0); // we have deleted all items new idx should be 0
    }
}
