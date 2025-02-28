use actix::{Actor, Handler, Message, SyncContext};
use diesel::{
    r2d2::{ConnectionManager, Pool},
};
use std::{fmt::Debug, marker::PhantomData};
use diesel::r2d2::R2D2Connection;

#[derive(Debug)]
pub(crate) struct Executor<C: 'static>(pub(crate) Pool<ConnectionManager<C>>)
where
    C: R2D2Connection;

impl<C> Actor for Executor<C>
where
    C: R2D2Connection,
{
    type Context = SyncContext<Self>;
}

pub(crate) struct Execute<F, C, R, E>(pub(crate) F, pub(crate) PhantomData<(C, R)>)
where
    R: 'static + Send,
    E: 'static + Debug + Send + Sync,
    C: R2D2Connection,
    F: FnOnce(&mut C) -> Result<R, E>;

impl<F, C, R, E> Message for Execute<F, C, R, E>
where
    R: Send,
    E: Debug + Send + Sync,
    C: R2D2Connection,
    F: FnOnce(&mut C) -> Result<R, E>,
{
    type Result = Result<Result<R, E>, r2d2::Error>;
}

impl<F, C, R, E> Handler<Execute<F, C, R, E>> for Executor<C>
where
    R: Send,
    E: Debug + Send + Sync,
    C: R2D2Connection,
    F: FnOnce(&mut C) -> Result<R, E>,
{
    type Result = Result<Result<R, E>, r2d2::Error>;

    fn handle(&mut self, msg: Execute<F, C, R, E>, _: &mut Self::Context) -> Self::Result {
        let mut conn = match self.0.get() {
            Ok(conn) => conn,
            Err(err) => return Err(err),
        };

        Ok((msg.0)(&mut *conn))
    }
}
