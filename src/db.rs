use crate::{
    executor::{Execute, Executor},
    AsyncError, Builder,
};
use actix::Addr;
use diesel::{
    r2d2::{ConnectionManager, Pool},
};
use futures::{Future, FutureExt};
use once_cell::sync::OnceCell;
use std::{fmt::Debug, marker::PhantomData, sync::Arc};
use diesel::r2d2::R2D2Connection;

pub struct Database<C: 'static>
where
    C: R2D2Connection,
{
    pub(crate) cell: Arc<OnceCell<Addr<Executor<C>>>>,
    pub(crate) pool: Pool<ConnectionManager<C>>,
    pub(crate) init: fn(Pool<ConnectionManager<C>>) -> Addr<Executor<C>>,
}

impl<C> Clone for Database<C>
where
    C: R2D2Connection,
{
    fn clone(&self) -> Self {
        Database {
            cell: self.cell.clone(),
            init: self.init,
            pool: self.pool.clone(),
        }
    }
}

impl<C> Database<C>
where
    C: R2D2Connection,
{
    #[inline]
    pub fn open(url: impl Into<String>) -> Database<C> {
        Self::builder().open(url)
    }

    #[inline]
    pub fn builder() -> Builder<C> {
        Builder {
            phantom: PhantomData,
            pool_max_size: None,
            pool_min_idle: None,
            pool_max_lifetime: None,
            on_acquire: None,
            on_release: None,
        }
    }

    /// Executes the given function inside a database transaction.
    #[inline]
    pub fn transaction<F, R, E>(&self, f: F) -> impl Future<Output=Result<R, AsyncError<E>>>
    where
        F: 'static + FnOnce(&C) -> Result<R, E> + Send,
        R: 'static + Send,
        E: 'static + From<diesel::result::Error> + Debug + Send + Sync,
    {
        self.get(move |conn| conn.transaction(move |conn| f(conn)))
    }

    /// Executes the given function with a connection retrieved from the pool.
    ///
    /// This is non-blocking and uses a `SyncArbiter` to provide a thread pool.
    pub fn get<F, R, E>(&self, f: F) -> impl Future<Output = Result<R, AsyncError<E>>>
    where
        F: 'static + FnOnce(&mut C) -> Result<R, E> + Send,
        R: 'static + Send,
        E: 'static + Debug + Send + Sync,
    {
        self.cell
            .get_or_init(|| (self.init)(self.pool.clone()))
            .send(Execute(f, PhantomData))
            .then(|res| async move {
                match res {
                    Ok(res) => match res {
                        Ok(res) => match res {
                            Ok(value) => Ok(value),
                            Err(err) => Err(AsyncError::Execute(err)),
                        },

                        Err(err) => Err(AsyncError::Timeout(err)),
                    },

                    Err(err) => Err(AsyncError::Delivery(err)),
                }
            })
    }
}
