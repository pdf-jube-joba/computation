pub trait SubSet<T>
where
    Self: Sized,
{
    fn into_super(self) -> T;
    fn from_super(s: &T) -> Option<Self>;
}
