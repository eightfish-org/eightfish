//! RPC interface for the transaction payment module.

use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
use openforum_runtime_api::OpenForumApi as OpenForumRuntimeApi;

#[rpc(client, server)]
pub trait OpenForumRpc<BlockHash> {
    #[method(name = "openforum_check_pair_list")]
	fn check_pair_list(&self, 
                       at: Option<BlockHash>,
                       model: Vec<u8>, 
                       pair_list: Vec<(Vec<u8>, Vec<u8>)>) -> RpcResult<bool>;
}

/// A struct that implements the `OpenForumRpc`.
pub struct OpenForum<C, M> {
	// If you have more generics, no need to SumStorage<C, M, N, P, ...>
	// just use a tuple like SumStorage<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> OpenForum<C, M> {
	/// Create new `OpenForum` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

/// Error type of this RPC api.
pub enum Error {
 	/// The transaction was not decodable.
 	DecodeError,
 	/// The call to runtime failed.
 	RuntimeError,
}

impl From<Error> for i32 {
 	fn from(e: Error) -> i32 {
 		match e {
 			Error::RuntimeError => 1,
 			Error::DecodeError => 2,
 		}
 	}
}

#[async_trait]
impl<C, Block> OpenForumRpcServer<<Block as BlockT>::Hash> for OpenForum<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: OpenForumRuntimeApi<Block>,
{
	fn check_pair_list(&self, 
                        at: Option<<Block as BlockT>::Hash>, 
                        model: Vec<u8>, 
                        pair_list: Vec<(Vec<u8>, Vec<u8>)>) -> RpcResult<bool> {

		let runtime_api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = runtime_api.check_pair_list(&at, model, pair_list);
        runtime_api_result.map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to query dispatch info.",
                    Some(e.to_string()),
                    ))
                .into()
        })
	}
}


