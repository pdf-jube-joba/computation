pub trait SubSet: Sized {
    type Super;
    fn into_super(self) -> Self::Super;
    fn from_super(s: &Self::Super) -> Option<Self>;
}
