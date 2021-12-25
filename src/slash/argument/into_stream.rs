#[doc(hidden)]
pub struct IntoStreamWrap<'a, T>(pub &'a T);

#[doc(hidden)]
pub trait IntoStream<T> {
    type Output;
    fn converter(self, x: T) -> Self::Output;
}

impl<T: IntoIterator> IntoStream<T> for &IntoStreamWrap<'_, T> {
    type Output = futures::stream::Iter<T::IntoIter>;
    fn converter(self, iter: T) -> Self::Output {
        futures::stream::iter(iter)
    }
}

impl<T: futures::Stream> IntoStream<T> for &&IntoStreamWrap<'_, T> {
    type Output = T;
    fn converter(self, stream: T) -> Self::Output {
        stream
    }
}

// Takes an expression that is either an IntoIterator or a Stream, and converts it to a Stream
#[doc(hidden)]
#[macro_export]
macro_rules! into_stream {
    ($e:expr) => {
        match $e {
            value => {
                use $crate::IntoStream as _;
                (&&$crate::IntoStreamWrap(&value)).converter()(value)
            }
        }
    };
}
