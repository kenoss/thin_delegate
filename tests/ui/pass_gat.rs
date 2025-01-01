// thin_delegate supports GATs.

#[thin_delegate::register]
trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

struct VecWithIndex<T> {
    xs: Vec<T>,
    i: usize,
}

impl<T> VecWithIndex<T> {
    fn new(xs: Vec<T>) -> Self {
        Self { xs, i: 0 }
    }
}

impl<T> LendingIterator for VecWithIndex<T> {
    type Item<'a> = &'a T
    where
        T: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        if self.xs.len() <= self.i {
            None
        } else {
            let i = self.i;
            self.i += 1;
            Some(&self.xs[i])
        }
    }
}

#[thin_delegate::register]
struct Wrapped<T>(VecWithIndex<T>);

#[thin_delegate::fill_delegate]
impl<T> LendingIterator for Wrapped<T> {
    type Item<'a> = &'a T
    where
        T: 'a;
}

fn main() {
    let xs = vec![0, 1, 2];
    let xs = VecWithIndex::new(xs);
    let mut xs = Wrapped(xs);

    assert_eq!(xs.next(), Some(&0));
    assert_eq!(xs.next(), Some(&1));
    assert_eq!(xs.next(), Some(&2));
    assert_eq!(xs.next(), None);
}
