#[doc(hidden)]
pub struct ConvertableIntoStream<T>(pub Option<T>);

#[doc(hidden)]
pub trait ConvertIntoStream {}

macro_rules! into_stream {
    ($e:expr) => {{
        use $crate::ConvertIntoStream;
        $crate::ConvertableIntoStream(Some($e)).convert()
    }};
}
