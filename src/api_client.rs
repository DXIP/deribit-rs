// mod account;
// mod authentication;
// mod session_management;
// mod subscription;
// mod support;
// mod trading;

use crate::errors::Result;
use crate::models::{JSONRPCRequest, JSONRPCResponse, Request};
use crate::WSStream;
use futures::channel::{mpsc, oneshot};
use futures::compat::Compat01As03Sink;
use futures::task::Waker;
use futures::{Future, Poll, SinkExt};
use futures01::stream::SplitSink as SplitSink01;
use log::trace;
use pin_utils::unsafe_pinned;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_value, to_string};
use std::marker::PhantomData;
use std::pin::Pin;
use tungstenite::Message;

type SplitWSCompatStream = Compat01As03Sink<SplitSink01<WSStream>, Message>;

pub struct DeribitAPIClient {
    wstx: SplitWSCompatStream,
    waiter_tx: mpsc::Sender<(i64, oneshot::Sender<Result<JSONRPCResponse>>)>,

    id: i64,
}

impl DeribitAPIClient {
    pub(crate) fn new(
        wstx: SplitWSCompatStream,
        waiter_tx: mpsc::Sender<(i64, oneshot::Sender<Result<JSONRPCResponse>>)>,
    ) -> DeribitAPIClient {
        DeribitAPIClient {
            wstx: wstx,
            waiter_tx: waiter_tx,

            id: 0,
        }
    }

    pub async fn call_raw<'a, R>(
        &'a mut self,
        request: R,
    ) -> Result<DeribitAPICallRawResult<R::Response>>
    where
        R: Request + Serialize + 'a,
    {
        let (waiter_tx, waiter_rx) = oneshot::channel();
        let req = JSONRPCRequest {
            id: self.id,
            method: R::METHOD.into(),
            params: request,
        };
        self.id += 1;

        let payload = to_string(&req)?;
        trace!("[Deribit] Request: {}", payload);
        await!(self.wstx.send(Message::Text(payload)))?;
        await!(self.waiter_tx.send((req.id, waiter_tx)))?;
        Ok(DeribitAPICallRawResult::new(waiter_rx))
    }

    pub async fn call<'a, R>(&'a mut self, request: R) -> Result<DeribitAPICallResult<R::Response>>
    where
        R: Request + Serialize + 'a,
    {
        let resp: DeribitAPICallRawResult<R::Response> = await!(self.call_raw(request))?;
        Ok(DeribitAPICallResult::new(resp))
    }
}

pub struct DeribitAPICallRawResult<R> {
    rx: oneshot::Receiver<Result<JSONRPCResponse>>,
    _ty: PhantomData<R>,
}

impl<R> DeribitAPICallRawResult<R> {
    pub(crate) fn new(rx: oneshot::Receiver<Result<JSONRPCResponse>>) -> Self {
        DeribitAPICallRawResult {
            rx: rx,
            _ty: PhantomData,
        }
    }
    unsafe_pinned!(rx: oneshot::Receiver<Result<JSONRPCResponse>>);
}

impl<R> Future for DeribitAPICallRawResult<R>
where
    R: DeserializeOwned,
{
    type Output = Result<JSONRPCResponse<R>>;
    fn poll(self: Pin<&mut Self>, waker: &Waker) -> Poll<Result<JSONRPCResponse<R>>> {
        self.rx().poll(waker).map(|result| {
            let resp = result??;
            let result = from_value(
                resp.result
                    .expect("result of JSONRPCResponse cannot be None"),
            )?;
            Ok(JSONRPCResponse {
                jsonrpc: resp.jsonrpc,
                id: resp.id,
                testnet: resp.testnet,
                error: None,
                result: Some(result),
                us_in: resp.us_in,
                us_out: resp.us_out,
                us_diff: resp.us_diff,
            })
        })
    }
}

pub struct DeribitAPICallResult<R> {
    inner: DeribitAPICallRawResult<R>,
}

impl<R> DeribitAPICallResult<R> {
    pub(crate) fn new(inner: DeribitAPICallRawResult<R>) -> Self {
        DeribitAPICallResult { inner: inner }
    }
    unsafe_pinned!(inner: DeribitAPICallRawResult<R>);
}

impl<R> Future for DeribitAPICallResult<R>
where
    R: DeserializeOwned,
{
    type Output = Result<R>;
    fn poll(self: Pin<&mut Self>, waker: &Waker) -> Poll<Result<R>> {
        self.inner().poll(waker).map(|result| {
            let resp = result?;
            Ok(resp
                .result
                .expect("result of JSONRPCResponse cannot be None"))
        })
    }
}
