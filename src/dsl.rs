use crate::{AsyncError, Database};
use diesel::{
    dsl::Limit,
    query_dsl::{limit_dsl::LimitDsl, load_dsl::ExecuteDsl, LoadQuery},
    result::{Error, OptionalExtension},
    Connection, RunQueryDsl,
};
use diesel::r2d2::R2D2Connection;
use futures::future::BoxFuture;

type AsyncDslFuture<I> = BoxFuture<'static, Result<I, AsyncError<Error>>>;

pub trait AsyncRunQueryDsl<Conn>: RunQueryDsl<Conn>
where
    Conn: R2D2Connection,
{
    fn execute_async(
        self,
        db: &Database<Conn>,
    ) -> AsyncDslFuture<usize>
    where
        Conn: Connection,
        Self: ExecuteDsl<Conn>;

    fn load_async<'a, U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> AsyncDslFuture<Vec<U>>
    where
        U: Send,
        Self: LoadQuery<'a, Conn, U>;

    fn get_result_async<'a, U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> AsyncDslFuture<U>
    where
        U: Send,
        Self: LoadQuery<'a, Conn, U>;

    fn get_optional_result_async<'a, U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> AsyncDslFuture<Option<U>>
    where
        U: Send,
        Self: LoadQuery<'a, Conn, U>;

    fn get_results_async<'a, U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> AsyncDslFuture<Vec<U>>
    where
        U: Send,
        Self: LoadQuery<'a, Conn, U>;

    fn first_async<'a, U: 'static>(
        self,
        db: &Database<Conn>,
    ) ->  AsyncDslFuture<U>
    where
        U: Send,
        Self: LimitDsl,
        Limit<Self>: LoadQuery<'a, Conn, U>;
}

impl<T: 'static, Conn> AsyncRunQueryDsl<Conn> for T
where
    T: RunQueryDsl<Conn> + Send,
    Conn: R2D2Connection,
{
    #[inline]
    fn execute_async(
        self,
        db: &Database<Conn>,
    ) -> AsyncDslFuture<usize>
    where
        Conn: Connection,
        Self: ExecuteDsl<Conn>,
    {
        Box::pin(db.get(move |conn| self.execute(conn)))
    }

    #[inline]
    fn load_async<'a, U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> AsyncDslFuture<Vec<U>>
    where
        U: Send,
        Self: LoadQuery<'a, Conn, U>,
    {
        Box::pin(db.get(move |conn| self.load(conn)))
    }

    #[inline]
    fn get_result_async<'a, U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> AsyncDslFuture<U>
    where
        U: Send,
        Self: LoadQuery<'a, Conn, U>,
    {
        Box::pin(db.get(move |conn| self.get_result(conn)))
    }

    #[inline]
    fn get_optional_result_async<'a, U: 'static>(
        self,
        db: &Database<Conn>,
    ) ->  AsyncDslFuture<Option<U>>
    where
        U: Send,
        Self: LoadQuery<'a, Conn, U>,
    {
        Box::pin(db.get(move |conn| self.get_result(conn).optional()))
    }

    #[inline]
    fn get_results_async<'a, U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> AsyncDslFuture<Vec<U>>
    where
        U: Send,
        Self: LoadQuery<'a, Conn, U>,
    {
        Box::pin(db.get(move |conn| self.get_results(conn)))
    }

    #[inline]
    fn first_async<'a, U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> AsyncDslFuture<U>
    where
        U: Send,
        Self: LimitDsl,
        Limit<Self>: LoadQuery<'a, Conn, U>,
    {
        Box::pin(db.get(move |conn| self.first(conn)))
    }
}
