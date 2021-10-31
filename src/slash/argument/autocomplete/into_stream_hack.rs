#[doc(hidden)]
pub struct IntoStreamWrap<'a, T>(pub &'a T);

#[doc(hidden)]
pub trait ConvertStreamFrom<T> {
    type Output;
    fn converter(self) -> fn(T) -> Self::Output;
}

impl<T: IntoIterator> ConvertStreamFrom<T> for &IntoStreamWrap<'_, T> {
    type Output = futures::stream::Iter<T::IntoIter>;
    fn converter(self) -> fn(T) -> Self::Output {
        |iter| futures::stream::iter(iter)
    }
}

impl<T: futures::Stream> ConvertStreamFrom<T> for &&IntoStreamWrap<'_, T> {
    type Output = T;
    fn converter(self) -> fn(T) -> Self::Output {
        |stream| stream
    }
}

// Takes an expression that is either an IntoIterator or a Stream, and converts it to a Stream
#[doc(hidden)]
#[macro_export]
macro_rules! into_stream {
    ($e:expr) => {
        match $e {
            value => {
                use $crate::ConvertStreamFrom;
                (&&$crate::IntoStreamWrap(&value)).converter()(value)
            }
        }
    };
}
