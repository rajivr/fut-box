use futures_core::stream::Stream;

#[doc(hidden)]
pub fn assert_is_unpin_stream<S: Stream + Unpin>(_: &mut S) {}

/// Assert that the next poll to the provided stream will return
/// [`Poll::Pending`](futures_core::task::Poll::Pending).
///
/// # Examples
///
/// ```
/// use futures::stream;
/// use futures_test::future::FutureTestExt;
/// use futures_test::{
///     assert_stream_pending, assert_stream_next, assert_stream_done,
/// };
/// use futures::pin_mut;
///
/// let stream = stream::once((async { 5 }).pending_once());
/// pin_mut!(stream);
///
/// assert_stream_pending!(stream);
/// assert_stream_next!(stream, 5);
/// assert_stream_done!(stream);
/// ```
#[macro_export]
macro_rules! assert_stream_pending {
    ($stream:expr) => {{
        let mut stream = &mut $stream;
        $crate::assert::assert_is_unpin_stream(stream);
        let stream = $crate::std_reexport::pin::Pin::new(stream);
        let mut cx = $crate::task::noop_context();
        let poll = $crate::futures_core_reexport::stream::Stream::poll_next(
            stream, &mut cx,
        );
        if poll.is_ready() {
            panic!("assertion failed: stream is not pending");
        }
    }};
}

/// Assert that the next poll to the provided stream will return
/// [`Poll::Ready`](futures_core::task::Poll::Ready) with the provided item.
///
/// # Examples
///
/// ```
/// use futures::stream;
/// use futures_test::future::FutureTestExt;
/// use futures_test::{
///     assert_stream_pending, assert_stream_next, assert_stream_done,
/// };
/// use futures::pin_mut;
///
/// let stream = stream::once((async { 5 }).pending_once());
/// pin_mut!(stream);
///
/// assert_stream_pending!(stream);
/// assert_stream_next!(stream, 5);
/// assert_stream_done!(stream);
/// ```
#[macro_export]
macro_rules! assert_stream_next {
    ($stream:expr, $item:expr) => {{
        let mut stream = &mut $stream;
        $crate::assert::assert_is_unpin_stream(stream);
        let stream = $crate::std_reexport::pin::Pin::new(stream);
        let mut cx = $crate::task::noop_context();
        match $crate::futures_core_reexport::stream::Stream::poll_next(stream, &mut cx) {
            $crate::futures_core_reexport::task::Poll::Ready(Some(x)) => {
                assert_eq!(x, $item);
            }
            $crate::futures_core_reexport::task::Poll::Ready(None) => {
                panic!("assertion failed: expected stream to provide item but stream is at its end");
            }
            $crate::futures_core_reexport::task::Poll::Pending => {
                panic!("assertion failed: expected stream to provide item but stream wasn't ready");
            }
        }
    }}
}

/// Assert that the next poll to the provided stream will return an empty
/// [`Poll::Ready`](futures_core::task::Poll::Ready) signalling the
/// completion of the stream.
///
/// # Examples
///
/// ```
/// use futures::stream;
/// use futures_test::future::FutureTestExt;
/// use futures_test::{
///     assert_stream_pending, assert_stream_next, assert_stream_done,
/// };
/// use futures::pin_mut;
///
/// let stream = stream::once((async { 5 }).pending_once());
/// pin_mut!(stream);
///
/// assert_stream_pending!(stream);
/// assert_stream_next!(stream, 5);
/// assert_stream_done!(stream);
/// ```
#[macro_export]
macro_rules! assert_stream_done {
    ($stream:expr) => {{
        let mut stream = &mut $stream;
        $crate::assert::assert_is_unpin_stream(stream);
        let stream = $crate::std_reexport::pin::Pin::new(stream);
        let mut cx = $crate::task::noop_context();
        match $crate::futures_core_reexport::stream::Stream::poll_next(stream, &mut cx) {
            $crate::futures_core_reexport::task::Poll::Ready(Some(_)) => {
                panic!("assertion failed: expected stream to be done but had more elements");
            }
            $crate::futures_core_reexport::task::Poll::Ready(None) => {}
            $crate::futures_core_reexport::task::Poll::Pending => {
                panic!("assertion failed: expected stream to be done but was pending");
            }
        }
    }}
}
