//! Small hacky macro to convert any value into a Stream, where the value can be an IntoIterator
//! or a Stream. Used for the return value of autocomplete callbacks

#[doc(hidden)]
pub struct IntoStreamWrap<'a, T>(pub &'a T);

#[doc(hidden)]
pub trait IntoStream<T> {
    type Output;
    // Have to return a callback instead of simply taking a parameter because we're moving T in,
    // but self still points into it (`cannot move out of _ because it is borrowed`)
    fn converter(self) -> fn(T) -> Self::Output;
}

impl<T: IntoIterator> IntoStream<T> for &IntoStreamWrap<'_, T> {
    type Output = futures::stream::Iter<T::IntoIter>;
    fn converter(self) -> fn(T) -> Self::Output {
        |iter| futures::stream::iter(iter)
    }
}

impl<T: futures::Stream> IntoStream<T> for &&IntoStreamWrap<'_, T> {
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
                use $crate::IntoStream as _;
                (&&$crate::IntoStreamWrap(&value)).converter()(value)
            }
        }
    };
}
