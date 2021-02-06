
use core::{mem,slice};
use generic_array::{ArrayLength,GenericArray};

#[doc(hidden)]
pub enum Entry<T> {
    Vacant(usize),
    Occupied(T),
}

impl<A> crate::i::Slab<A> {
    pub const fn new()-> Self {
        Self {
            entries:crate::i::Vec::new(),
            len:0,
            next:0
        }
    }
}

pub struct Slab<T,N>(#[doc(hidden)] pub crate::i::Slab<GenericArray<Entry<T>,N>>)
where
    N: ArrayLength<Entry<T>>;

impl<T,N> Slab<T,N>
where
    N: ArrayLength<Entry<T>>
{
    pub fn new()-> Self {
        Slab(crate::i::Slab::new())
    }

    pub fn insert(&mut self,val:T) -> Result<usize,T> {
        let key = self.0.next;
        self.insert_at(key,val)?;
        Ok(key)
    }

    pub fn insert_at(&mut self,key:usize,val:T) -> Result<(),T> {
        self.0.len += 1;
        if key == self.0.entries.len {
            self.0.entries.push(Entry::Occupied(val)).map_err(|entry| {
                if let Entry::Occupied(val) = entry {
                    val
                } else {
                    unreachable!()
                }
            })?;
            self.0.next = key + 1;
        } else {
            let prev = mem::replace(
                &mut self.0.entries.as_mut_slice()[key],
                Entry::Occupied(val),
            );

            match prev {
                Entry::Vacant(next) => {
                    self.0.next = next;
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    pub fn remove(&mut self,key:usize) -> T {
        let prev = mem::replace(
            &mut self.0.entries.as_mut_slice()[key],
            Entry::Vacant(self.0.next)
        );

        match prev {
            Entry::Occupied(val) => {
                self.0.len -= 1;
                self.0.next =key;
                val
            }
            _=> {
                self.0.entries.as_mut_slice()[key] = prev;
                panic!("invalid key")
            }
        }
    }

    pub fn remove_safe(&mut self,key:usize) -> Option<T> {
        let prev = mem::replace(
            &mut self.0.entries.as_mut_slice()[key],
            Entry::Vacant(self.0.next)
        );

        match prev {
            Entry::Occupied(val) => {
                self.0.len -= 1;
                self.0.next =key;
                Some(val)
            }
            _=> {
                self.0.entries.as_mut_slice()[key] = prev;
                None
            }
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_,T> {
        IterMut {
            entries: self.0.entries.as_mut_slice().iter_mut(),
            curr:0
        }
    }
}
impl<T,N> Default for Slab<T,N>
where
    N: ArrayLength<Entry<T>>
{
    fn default() -> Self {
        Self::new()
    }
}


pub struct IterMut<'a,T> {
    entries:slice::IterMut<'a,Entry<T>>,
    curr:usize
}

impl <'a,T> Iterator for IterMut<'a,T> {
    type Item = (usize,&'a mut T);

    fn next(&mut self) -> Option<(usize,&'a mut T)> {
        while let Some(entry) = self.entries.next() {
            let curr = self.curr;
            self.curr += 1;

            if let Entry::Occupied(ref mut v) = *entry {
                return Some((curr,v))
            }
        }
        None
    }
}