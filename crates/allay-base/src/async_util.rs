//! Async conversion traits
#[async_trait::async_trait]
pub trait AsyncFrom<T>: Sized {
    async fn async_from(value: T) -> Self;
}

#[async_trait::async_trait]
pub trait AsyncInto<T> {
    async fn async_into(self) -> T;
}

#[async_trait::async_trait]
impl<T, U> AsyncInto<U> for T
where
    U: AsyncFrom<T>,
    T: Send,
{
    async fn async_into(self) -> U {
        U::async_from(self).await
    }
}
