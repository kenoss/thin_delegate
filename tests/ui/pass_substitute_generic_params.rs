// This test checks that thin_delegate handles type substitution `T <- String` for `<T as Writable>`
// in the function singnature.

trait Writable {
    type Buffer<'a>: WritableBuffer + 'a
    where
        Self: 'a;

    fn get_buffer(&mut self) -> Self::Buffer<'_>;
}

trait WritableBuffer {
    fn write(&mut self, s: &str);
}

#[thin_delegate::register]
trait Hello<T>
where
    T: Writable,
{
    fn hello(&self, buf: &mut <T as Writable>::Buffer<'_>);
}

#[thin_delegate::register]
struct Hoge(String);

#[thin_delegate::fill_delegate]
impl Hello<String> for Hoge {}

impl Writable for String {
    type Buffer<'a> = StringBuffer<'a>;

    fn get_buffer(&mut self) -> StringBuffer<'_> {
        StringBuffer { s: self }
    }
}

struct StringBuffer<'a> {
    s: &'a mut String,
}

impl WritableBuffer for StringBuffer<'_> {
    fn write(&mut self, s: &str) {
        self.s.push_str(s);
    }
}

impl<T> Hello<T> for String
where
    T: Writable,
{
    fn hello(&self, buf: &mut <T as Writable>::Buffer<'_>) {
        buf.write(&self);
    }
}

fn main() {
    let mut w = String::new();

    let hoge = Hoge("hello".to_string());
    hoge.hello(&mut w.get_buffer());
    assert_eq!(w, "hello");
}
