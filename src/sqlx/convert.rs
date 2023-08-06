use std::{vec::IntoIter, task::Poll};

use futures::{Stream, Future, StreamExt};

pub enum Streamed<F, I> {
    Fut(F),
    Yelded(IntoIter<I>),
}

impl<F, I, E> Streamed<F, I> where F: Future<Output = Result<Vec<I>, E>>{
    pub fn new(f: F) -> Self {
        Self::Fut(f)
    }
}

impl <F, I, E> Stream for Streamed<F, I> where F: Future<Output = Result<Vec<I>, E>> + Unpin, I: Unpin, E: Unpin {
    type Item = Result<I, E>;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        unsafe {
            let unpin = self.get_unchecked_mut();
            match unpin {
                Streamed::Fut(f) => match std::pin::Pin::new(f).poll(cx) {
                    Poll::Ready(Ok(v)) => {
                        let mut iter = v.into_iter();
                        let item = iter.next().map(|i|Ok(i));
                        *unpin = Streamed::Yelded(iter);
                        Poll::Ready(item)
                    },
                    Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
                    Poll::Pending => Poll::Pending,
                },
                Streamed::Yelded(iter) => {
                    Poll::Ready(iter.next().map(|i|Ok(i)))
                },
            }
        }
    }
}

#[tokio::test]
async fn xx() {
    let f = Box::pin(async {
        Ok::<_, ()>(vec![1,2,3])
    });
    let mut stream = Box::pin(Streamed::new(f));
    while let Some(Ok(x)) = stream.next().await {
        println!("{x}");
    }
}