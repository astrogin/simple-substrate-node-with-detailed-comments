//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

// is used to create a thread-safe reference-counted pointer to the RpcModule.
use std::sync::Arc;
// is a struct representing a JSON-RPC module that can be registered with a server to expose methods to remote clients.
use jsonrpsee::RpcModule;
// are types defined by the Substrate runtime used in this specific node implementation. 
// Block is an opaque type representing a block in the blockchain, while AccountId, Balance, 
// and Index are types used to represent account identifiers, token balances, and indices respectively.
use substrate_simple_runtime_with_comments::{opaque::Block, AccountId, Balance, Index};
// is a trait that defines the interface to interact with the transaction pool of a Substrate node. 
// This allows adding transactions to the pool, fetching transactions from the pool, and managing transaction fees.
use sc_transaction_pool_api::TransactionPool;
// is a trait that defines the interface for providing access to the runtime API of a 
// Substrate node. This trait allows external modules to obtain references to the runtime API and call runtime methods.
use sp_api::ProvideRuntimeApi;
// is a trait that defines the interface for building new blocks in the blockchain. 
// This allows external modules to propose new blocks to be added to the blockchain.
use sp_block_builder::BlockBuilder;
// are types related to the blockchain management in a Substrate node. HeaderBackend is a 
// trait that defines the interface to interact with the blockchain headers, while HeaderMetadata 
// represents the metadata associated with a block header. BlockChainError is an error type used when interacting with the blockchain.
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

// DenyUnsafe is a type alias that is used to enable or disable unsafe APIs in the Substrate RPC server. 
// When unsafe APIs are disabled, any attempt to use them will result in a MethodNotFound error response. 
// This is a security feature that helps prevent potential exploits and attacks on the Substrate node.
pub use sc_rpc_api::DenyUnsafe;

/// Full client dependencies.
pub struct FullDeps<C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all full RPC extensions.
/// This code defines a function create_full that creates an instance of a jsonrpsee::RpcModule and merges in 
/// the available RPC APIs. The function takes in a FullDeps struct that contains the necessary dependencies 
/// for the function to work correctly, such as a client to interact with the Substrate runtime, 
/// a transaction pool, and a DenyUnsafe value.
/// The function merges in the System API RPC and Transaction Payment RPC APIs into the RpcModule. 
/// Additionally, the function includes a comment that shows how to extend the RpcModule with custom APIs. 
/// The function returns the merged RpcModule instance wrapped in a Result that returns a 
/// Box<dyn std::error::Error + Send + Sync> as the error type, meaning the error can be any type that implements those traits.
pub fn create_full<C, P>(
	deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + 'static,
{
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	let mut module = RpcModule::new(());
	let FullDeps { client, pool, deny_unsafe } = deps;

	module.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client).into_rpc())?;

	// Extend this RPC with a custom API by using the following syntax.
	// `YourRpcStruct` should have a reference to a client, which is needed
	// to call into the runtime.
	// `module.merge(YourRpcTrait::into_rpc(YourRpcStruct::new(ReferenceToClient, ...)))?;`

	Ok(module)
}
