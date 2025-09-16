use std::cmp::Ordering;
use std::fmt;
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[repr(align(8))]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[must_use]
pub struct StackVec<T, const CAP: usize> {
    len: u8,
    elems: [T; CAP],
}

impl<T: fmt::Debug, const CAP: usize> fmt::Debug for StackVec<T, CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: Default + Copy, const CAP: usize> Default for StackVec<T, CAP> {
    fn default() -> Self {
        assert!(CAP <= u8::MAX as usize, "capacity too big");
        Self {
            len: 0,
            elems: [T::default(); CAP],
        }
    }
}

impl<T: Default + Copy, const CAP: usize> StackVec<T, CAP> {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn push(mut self, elem: T) -> Option<Self> {
        *self.elems.get_mut(self.len as usize)? = elem;
        self.len += 1;
        Some(self)
    }

    #[must_use]
    pub fn map<U: Default + Copy>(self, f: impl FnMut(T) -> U) -> StackVec<U, CAP> {
        StackVec::from_iter(self.into_iter().map(f)).unwrap()
    }
    #[must_use]
    pub fn retain_unsorted(mut self, mut f: impl FnMut(T) -> bool) -> StackVec<T, CAP> {
        if self.len == 0 {
            return self;
        }
        let mut i = 0;
        while i < self.len() {
            if f(self[i]) {
                i += 1;
            } else {
                self = self.swap_remove(i);
            }
        }
        self
    }

    #[must_use]
    pub fn extend(mut self, iter: impl IntoIterator<Item = T>) -> Option<Self> {
        let iter = iter.into_iter();

        let (lo, _) = iter.size_hint();
        if self.len as usize + lo > CAP {
            return None; // definitely won't fit
        }

        for elem in iter {
            self = self.push(elem)?;
        }
        Some(self)
    }

    pub fn from_iter(iter: impl IntoIterator<Item = T>) -> Option<Self> {
        Self::new().extend(iter)
    }

    #[must_use]
    pub fn swap_remove(mut self, index: usize) -> Self {
        self[index] = self[self.len() - 1];
        self.len -= 1;
        self
    }
}
impl<T, const CAP: usize> StackVec<T, CAP> {
    #[must_use]
    pub fn sorted_unstable(mut self) -> Self
    where
        T: Ord,
    {
        self.sort_unstable();
        self
    }
    #[must_use]
    pub fn sorted_unstable_by_key<K: Ord>(mut self, f: impl FnMut(&T) -> K) -> Self {
        self.sort_unstable_by_key(f);
        self
    }
    #[must_use]
    pub fn sorted_unstable_by(mut self, compare: impl FnMut(&T, &T) -> Ordering) -> Self {
        self.sort_unstable_by(compare);
        self
    }
}

impl<T, const CAP: usize> Deref for StackVec<T, CAP> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.elems[0..self.len as usize]
    }
}

impl<T, const CAP: usize> DerefMut for StackVec<T, CAP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elems[0..self.len as usize]
    }
}

impl<T, const CAP: usize> Index<usize> for StackVec<T, CAP> {
    type Output = T;

    #[track_caller]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len as usize, "index out of bounds");
        &self.elems[index]
    }
}

impl<T, const CAP: usize> IndexMut<usize> for StackVec<T, CAP> {
    #[track_caller]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len as usize, "index out of bounds");
        &mut self.elems[index]
    }
}

impl<T, const CAP: usize> IntoIterator for StackVec<T, CAP> {
    type Item = T;

    type IntoIter = std::iter::Take<std::array::IntoIter<T, CAP>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elems.into_iter().take(self.len as usize)
    }
}

impl<'a, T, const CAP: usize> IntoIterator for &'a StackVec<T, CAP> {
    type Item = &'a T;

    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.elems[0..self.len as usize].iter()
    }
}

impl<'a, T, const CAP: usize> IntoIterator for &'a mut StackVec<T, CAP> {
    type Item = &'a mut T;

    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.elems[0..self.len as usize].iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stackvec_merge() {
        let mut a = StackVec::<(u8, u8), 16>::new();
        a = a.push((1, 12)).unwrap();
        a = a.push((1, 6)).unwrap();
        a = a.push((2, 1)).unwrap();
        a = a.push((6, 92)).unwrap();
        a = a.push((1, 4)).unwrap();
        a = a.push((2, 9)).unwrap();
        a = a.push((3, 14)).unwrap();
        a = a.push((1, 3)).unwrap();
        a.sort_unstable();
    }

    #[test]
    fn test_stackvec_retain() {
        let mut a = StackVec::<u8, 16>::from_iter([9, 7, 10, 2, 8, 3, 1, 4, 6, 5]).unwrap();
        a = a.retain_unsorted(|x| x > 5);
        a.sort();
        assert_eq!(&*a, &[6, 7, 8, 9, 10]);

        let b = StackVec::<u8, 2>::from_iter([0, 10])
            .unwrap()
            .retain_unsorted(|x| x != 0);
        assert_eq!(&*b, &[10]);
    }
}
