use owning_ref::OwningHandle;
use std::cell::{Ref, RefCell};

// pub struct RefIter<'b, T>
// where
//     &'b T: IntoIterator,
// {
//     ref_: Ref<'b, T>,
//     iter: <&'b T as IntoIterator>::IntoIter,
// }

// impl<'b, T> Iterator for RefIter<'b, T>
// where
//     &'b T: IntoIterator,
// {
//     type Item = <&'b T as IntoIterator>::Item;
//     fn next(&mut self) -> Option<<RefIter<'b, T> as Iterator>::Item> {
//         self.iter.next()
//     }
// }

// pub fn ref_iter<'b, T>(refcell: &'b RefCell<T>) -> RefIter<'b, T>
// where
//     &'b T: IntoIterator,
// {
//     let ref_ = refcell.borrow();
//     let iter = ref_.into_iter();
//     RefIter { ref_, iter }
// }

pub struct RefIter<'b, T>(OwningHandle<Ref<'b, T>, Box<<&'b T as IntoIterator>::IntoIter>>)
where
    &'b T: IntoIterator;

impl<'b, T> RefIter<'b, T>
where
    &'b T: IntoIterator,
{
    pub fn new(ref_: Ref<'b, T>) -> RefIter<'b, T> {
        RefIter(OwningHandle::new_with_fn(ref_, |r: *const T| unsafe {
            Box::new((&*r).into_iter())
        }))
    }

    pub fn borrow(refcell: &'b RefCell<T>) -> RefIter<'b, T> {
        RefIter::new(refcell.borrow())
    }
}

impl<'b, T> Iterator for RefIter<'b, T>
where
    &'b T: IntoIterator,
{
    type Item = <&'b T as IntoIterator>::Item;
    fn next(&mut self) -> Option<<RefIter<'b, T> as Iterator>::Item> {
        self.0.next()
    }
}

pub fn refcell_iter<'b, T>(refcell: &'b RefCell<T>) -> RefIter<'b, T>
where
    &'b T: IntoIterator,
{
    let oh = OwningHandle::new_with_fn(refcell.borrow(), |r: *const T| unsafe {
        Box::new((&*r).into_iter())
    });
    RefIter(oh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_work() {
        let obj: RefCell<Vec<i32>> = RefCell::new((0..10).collect());

        let iter = RefIter::new(obj.borrow());

        for i in iter {
            println!("{:?}", i);
        }

        let iter = RefIter::borrow(&obj);

        for i in iter {
            println!("{:?}", i);
        }
    }
}
