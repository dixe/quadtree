use std::fmt;
use std::ops::{Index, IndexMut};


#[derive(Debug)]
pub struct FreeElement<T> {
    pub element: T,
    next: i32
}

pub struct FreeList<T> {
    pub data: Vec::<FreeElement<T>>,
    pub first_free: i32,
}


impl<T> FreeList<T> {

    pub fn new() -> Self {
        FreeList {
            data: Vec::new(),
            first_free: -1
        }
    }

    pub fn insert(&mut self, element: T) -> i32 {
        if self.first_free != -1 {
            let index = self.first_free;
            self.first_free = self.data[self.first_free as usize].next;
            self.data[index as usize].element = element;
            self.data[index as usize].next = -1;
            return index;
        }
        else {
            let fe = FreeElement {
                element,
                next: -1
            };

            self.data.push(fe);
            return (self.data.len() - 1) as i32;
        }

    }

    pub fn erase(&mut self, n: i32) {

        self.data[n as usize].next = self.first_free;
        self.first_free = n;
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.first_free = -1;

    }

    pub fn range(&self) -> i32 {
        (self.data.len() - 1)as i32
    }

    pub fn print(&self) -> String {

        let mut res = "".to_string();


        for (i, e) in self.data.iter().enumerate() {
            if e.next == -1 && self.first_free != i as i32 {
                res += &format!("{}, ", i);
            }
        }

        res
    }

}


impl<T> Index<i32> for FreeList<T> {
    type Output = FreeElement<T>;

    fn index<'a>(&'a self, i: i32) -> &'a FreeElement<T> {
        &self.data[i as usize]
    }
}


impl<T> IndexMut<i32> for FreeList<T> {
    fn index_mut<'a>(&'a mut self, i: i32) -> &'a mut FreeElement<T> {
        &mut self.data[i as usize]
    }
}


impl<T> fmt::Display for FreeList<T> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print())
    }
}

impl<T> fmt::Debug for FreeList<T> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
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
        assert_eq!(fl[idx2].element, 4);

        let idx3 = fl.insert(2);
        assert_eq!(idx1, idx3); // freeing idx2 is reused as idx3
        assert_eq!(fl[idx3].element, 2); // new element is retrived

        fl.erase(idx2);
        fl.erase(idx3);

        let idx = fl.insert(1);
        assert_eq!(idx, 0); // we have deleted all elements new idx should be 0
    }
}
