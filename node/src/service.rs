//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use substrate_simple_runtime_with_comments::{self, opaque::Block, RuntimeApi};
use sc_client_api::BlockBackend;
use sc_consensus_aura::{ImportQueueParams, SlotProportion, StartAuraParams};
pub use sc_executor::NativeElseWasmExecutor;
use sc_consensus_grandpa::SharedVoterState;
use sc_keystore::LocalKeystore;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager, WarpSyncParams};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use std::{sync::Arc, time::Duration};

// Our native executor instance.
pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
	/// Only enable the benchmarking host functions when we actually want to benchmark.
	#[cfg(feature = "runtime-benchmarks")]
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	/// Otherwise we only use the default Substrate host functions.
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		substrate_simple_runtime_with_comments::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		substrate_simple_runtime_with_comments::native_version()
	}
}

// In summary, this code defines a type alias called FullClient that implements the
// sc_service::TFullClient trait with three generic type parameters: Block, RuntimeApi,
// and NativeElseWasmExecutor<ExecutorDispatch>. The FullClient type is marked as pub(crate),
// meaning that it is visible within the current crate but not outside of it.
//
// In Substrate, TFullClient is a type alias for a struct that provides access to the full client
// implementation of a Substrate node. It combines several components of the node, such as the
// client itself, the block import mechanism, the transaction pool, the keystore container, and more, into a single struct.
//
// By using TFullClient, developers can easily access all the functionality of a Substrate node
// and build applications that interact with the blockchain. For example, they can use the
// client to submit transactions to the network, query the state of the blockchain, and manage
// accounts and keys. They can also use the block import mechanism to synchronize the local
// node with the rest of the network, and the transaction pool to manage pending transactions.
//
// Overall, TFullClient is a key component of a Substrate node, and provides a powerful and
// flexible interface for building decentralized applications on top of the Substrate framework.
pub(crate) type FullClient =
	sc_service::TFullClient<
		// This is a type that represents the blocks in the Substrate chain. The Block type is
		// generic and can be defined by the user.
		Block,
		// This is a type that represents the Substrate runtime API. The RuntimeApi type is also
		// generic and can be defined by the user.
		RuntimeApi,
		// This is a type that represents the Substrate executor. The NativeElseWasmExecutor is
		// an executor that can run both native and WASM code, while ExecutorDispatch is a
		// dispatch object that is used to determine which executor to use for a given block.
		NativeElseWasmExecutor<ExecutorDispatch>
	>;
// The backend is a component of a Substrate node that provides access to the blockchain data
// storage and retrieval functionalities. It is responsible for managing the storage of blocks
// and other data structures in the node's database, as well as providing access to that data
// to other components of the node, such as the client and the runtime.
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

pub fn new_partial(
	config: &Configuration,
) -> Result<
	sc_service::PartialComponents<
		FullClient,
		FullBackend,
		FullSelectChain,
		sc_consensus::DefaultImportQueue<Block, FullClient>,
		sc_transaction_pool::FullPool<Block, FullClient>,
		(
			sc_consensus_grandpa::GrandpaBlockImport<
				FullBackend,
				Block,
				FullClient,
				FullSelectChain,
			>,
			sc_consensus_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
			Option<Telemetry>,
		),
	>,
	ServiceError,
> {
	if config.keystore_remote.is_some() {
		return Err(ServiceError::Other("Remote Keystores are not supported.".into()))
	}

	// telemetry is used to collect and report various metrics related to the node's performance
	// and usage. This includes data such as block times, transaction throughput, storage usage,
	// and more. This information can be used to monitor the health of the node, identify potential
	// bottlenecks or issues, and make improvements to the node's configuration and performance.
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	// The NativeElseWasmExecutor is a type that is part of the sc_executor crate, and is used
	// to create an executor that can run both native code and WebAssembly code. In this
	// code, a new instance of NativeElseWasmExecutor is created, with the following arguments:
	// config.wasm_method: specifies the method that should be used to execute the WebAssembly code.
	// 	This can be set to either Compiled or Interpreted, depending on whether the WebAssembly code
	// 	should be compiled to native code or interpreted at runtime, respectively.
	// config.default_heap_pages: specifies the default number of pages to allocate for the heap when executing WebAssembly code.
	// config.max_runtime_instances: specifies the maximum number of runtime instances that can be created at any given time.
	// config.runtime_cache_size: specifies the size of the runtime cache.
	//
	// In a Substrate node, the runtime code is executed within a separate task managed by the
	// Tokio runtime. When a new block is received, for example, the Tokio runtime will spawn a
	// new task to validate that block. This task will use the executor to execute the runtime code
	// for the block, which may include executing transactions, updating state, and more. Once
	// the task is complete, the result is returned to the Tokio runtime, which can then continue processing the block.
	//
	// Overall, the combination of the NativeElseWasmExecutor and the Tokio runtime provides a
	// flexible and efficient way to execute runtime code in a Substrate node. The executor
	// itself is responsible for executing the code, while the Tokio runtime provides a scalable
	// and asynchronous framework for managing tasks and coordinating their execution.
	let executor = NativeElseWasmExecutor::<ExecutorDispatch>::new(
		config.wasm_method,
		config.default_heap_pages,
		config.max_runtime_instances,
		config.runtime_cache_size,
	);

	// client
	//  - off-chain operations/extensions
	//  - The code_substitutes configuration in a Substrate chain spec file allows for swapping
	// 		out the runtime code of the blockchain at specific block heights or block hashes.
	// 		This can be useful for deploying updates to the blockchain runtime without having to
	// 		perform a hard fork. By specifying a code substitute for a specific block height or
	// 		block hash, nodes that reach that block will start using the new runtime code instead
	// 		of the original code. This feature can be used to implement protocol upgrades, bug
	// 		fixes, or new features in a more seamless and gradual way.
	//
	// create backend-client (create DBs or connect if exists)
	//
	// KeystoreContainer is used to manage cryptographic keys and signing operations.
	// The KeystoreContainer is a container for Keystore instances, which represent individual
	// key stores that hold keys for signing transactions. The KeystoreContainer allows the node
	// to access keys stored in local files or in remote locations via RPC. It provides an
	// abstraction layer for accessing and managing keys, and ensures that only authorized
	// entities are able to access the keys. The KeystoreContainer is used throughout the
	// Substrate node to sign transactions, validate signatures, and manage keys for various
	// purposes. For example, it is used by the transaction pool to validate transactions and
	// by the consensus engine to sign block proposals.
	//
	// The task_manager in a Substrate node is responsible for managing asynchronous tasks that
	// are spawned by the node. These tasks can include things like network connections,
	// background processing, and other tasks that are required to keep the node running.
	// The task_manager is an instance of the TaskManager struct provided by the sc_service
	// crate. It provides a simple interface for spawning and managing tasks, and also
	// provides methods for graceful shutdown of the tasks when the node is shutting down.
	// In a Substrate node, the task_manager is used to manage a variety of tasks, including:
	// 	Network connections and communication with other nodes in the network
	// 	Background processing tasks, such as syncing the blockchain and validating transactions
	// 	Handling incoming RPC requests and other external events
	// 	Various other tasks required to keep the node running
	let (
		client,
		backend,
		keystore_container,
		task_manager
	) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;

	let client = Arc::new(client);

	// run telemetry in thread
	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	// In the context of a Substrate node, this code creates a new instance of the LongestChain
	// struct, which is used as the chain selection strategy for the consensus engine. The
	// LongestChain struct is provided by the sc_consensus crate and implements the ChainSelector
	// trait, which defines the logic for selecting the "best" chain in the blockchain based on chain length.
	// The LongestChain strategy selects the chain with the most blocks as the "best" chain, which
	// is a common approach used in many blockchain systems. This strategy works well for
	// blockchain networks where the probability of chain splits or forks is low, but can be
	// problematic in networks where forks are common or occur frequently.
	// The backend.clone() parameter passed to the LongestChain::new() method call is an
	// instance of the Backend trait, which provides the interface for reading and writing
	// data to and from the Substrate node's database backend. By passing in the backend
	// instance, the LongestChain struct is able to query the database to determine the length of
	// each chain and select the "best" one.
	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	// the code creates a new transaction pool using the BasicPool::new_full method. The transaction
	// pool is responsible for managing transactions and ensuring their inclusion in blocks.
	// The method takes several arguments:
	// config.transaction_pool.clone() - this is a configuration object for the transaction pool, which is cloned to create the new instance.
	// config.role.is_authority().into() - this determines whether the node is an authority node or not, and is used to configure certain aspects of the transaction pool's behavior.
	// config.prometheus_registry() - this is a Prometheus metrics registry used to track the transaction pool's performance.
	// task_manager.spawn_essential_handle() - this is a handle to the task manager, which is used to spawn new tasks within the Substrate runtime.
	// client.clone() - this is a handle to the Substrate client, which is used by the transaction pool to communicate with the rest of the node.
	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	// The Grandpa block import is responsible for importing new blocks from the network and
	// validating them before they are added to the chain.
	//
	// The block_import function takes several arguments including client, which is the
	// Substrate client instance, select_chain, which is the LongestChain instance used
	// for selecting the best chain, and telemetry, which is an optional telemetry handle
	// used for monitoring the Grandpa module's performance.
	//
	// The block_import function returns a tuple containing the grandpa_block_import and
	// grandpa_link. The grandpa_block_import is an instance of sc_consensus_grandpa::BlockImport,
	// which is responsible for importing and validating new blocks. The grandpa_link is an
	// instance of sc_consensus_grandpa::Link, which is used to interact with other Grandpa nodes on the network.
	let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
		client.clone(),
		&(client.clone() as Arc<_>),
		select_chain.clone(),
		telemetry.as_ref().map(|x| x.handle()),
	)?;

	let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

	// This code is used to create an import queue for the Aura consensus engine, which is one
	// of the consensus engines available in the Substrate blockchain framework.
	//
	// The import queue is responsible for importing new blocks into the chain by validating
	// them and applying any necessary changes to the blockchain state. The Aura import queue
	// is designed specifically for the Aura consensus algorithm, which uses a deterministic
	// algorithm to choose block producers and has a fixed block time.
	let import_queue =
		sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _>(ImportQueueParams {
			// block_import: A block import instance that is used to import blocks into the
			// blockchain. In this case, it is the grandpa_block_import instance, which is
			// used by the Grandpa finality algorithm to import finalized blocks.
			block_import: grandpa_block_import.clone(),
			// justification_import: A justification import instance that is used to
			// import justification data, which is used to finalize blocks. In this case,
			// it is a clone of the grandpa_block_import instance, which means that justification
			// data is imported in the same way as regular blocks.
			//
			// Justification data in a blockchain refers to the proof of the finality of a
			// block. It is used to prove that a block has been finalized by a supermajority
			// of the validators in the network, meaning that it cannot be reverted without a
			// significant portion of the validators agreeing to revert it.
			//
			// Justification data can be used for various purposes, such as:
			//
			// Light client verification: Justification data can be used by light clients to
			// verify that a block has been finalized without downloading the entire blockchain.
			//
			// Fork choice: Justification data can be used by nodes to choose which chain
			// to follow in the case of a fork.
			//
			// Security: Justification data provides a security guarantee that a block has been
			// finalized by a supermajority of validators, which makes it much harder to revert.
			//
			// For example, in the Polkadot network, justification data is used to guarantee the
			// finality of relay chain blocks, which contain the finality proofs of parachain
			// blocks. This allows parachains to trust that their blocks have been finalized and are secure.
			justification_import: Some(Box::new(grandpa_block_import.clone())),
			// client: A reference to the Substrate client that is used to interact with the network.
			client: client.clone(),
			// create_inherent_data_providers: A closure that creates the inherent data
			// providers for new blocks. In this case, it creates an inherent data provider
			// for the current timestamp and a slot duration.
			create_inherent_data_providers: move |_, ()| async move {
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

				let slot =
					sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
						*timestamp,
						slot_duration,
					);

				Ok((slot, timestamp))
			},
			// spawner: A handle to the task spawner that is used to spawn new tasks.
			spawner: &task_manager.spawn_essential_handle(),
			// registry: A Prometheus registry that is used to store metrics for the import queue.
			registry: config.prometheus_registry(),
			// check_for_equivocation: A flag that determines whether the import queue should
			// check for equivocation, which is when a block producer creates multiple conflicting blocks.
			check_for_equivocation: Default::default(),
			// telemetry: A handle to the telemetry worker that is used to report metrics about the import queue.
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			// compatibility_mode: A flag that determines whether the import queue
			// should be run in compatibility mode, which is used when upgrading a
			// blockchain from a previous version of Substrate.
			compatibility_mode: Default::default(),
		})?;

	Ok(sc_service::PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (grandpa_block_import, grandpa_link, telemetry),
	})
}

fn remote_keystore(_url: &String) -> Result<Arc<LocalKeystore>, &'static str> {
	// FIXME: here would the concrete keystore be built,
	//        must return a concrete type (NOT `LocalKeystore`) that
	//        implements `CryptoStore` and `SyncCryptoStore`
	Err("Remote Keystore not supported.")
}

/// Builds a new service for a full client.
pub fn new_full(mut config: Configuration) -> Result<TaskManager, ServiceError> {
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		mut keystore_container,
		select_chain,
		transaction_pool,
		other: (block_import, grandpa_link, mut telemetry),
	} = new_partial(&config)?;

	// In summary, this code snippet checks if a remote keystore is configured for the Substrate node,
	// sets it up if it is, and returns an error if the setup fails.
	if let Some(url) = &config.keystore_remote {
		match remote_keystore(url) {
			Ok(k) => keystore_container.set_remote_keystore(k),
			Err(e) =>
				return Err(ServiceError::Other(format!(
					"Error hooking up remote keystore for {}: {}",
					url, e
				))),
		};
	}
	// The resulting grandpa_protocol_name is a human-readable string that represents the
	// Grandpa protocol's name for the given chain. This name is used for reporting and logging purposes.
	let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
		&client.block_hash(0).ok().flatten().expect("Genesis block exists; qed"),
		&config.chain_spec,
	);

	// the extra_sets field is being used to add a new Grandpa peers set configuration to the
	// network's extra_sets array. This configuration is based on the grandpa_protocol_name,
	// which is the name of the Grandpa protocol standard used in the network.
	config
		.network
		.extra_sets
		.push(sc_consensus_grandpa::grandpa_peers_set_config(grandpa_protocol_name.clone()));

	// This code is creating a new NetworkProvider instance that will be used to provide the
	// warp proof data to the grandpa finality module.
	//
	// The NetworkProvider will be wrapped in an Arc and used by the grandpa finality
	// module to fetch warp proof data during the finality process.
	let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
		backend.clone(),
		// grandpa_link.shared_authority_set().clone(): This is the shared authority set used by
		// the grandpa finality module. It provides the network provider with access to the
		// grandpa authority set.
		grandpa_link.shared_authority_set().clone(),
		Vec::default(),
	));

	// This code is used to build the network configuration for a Substrate node. It
	// calls the build_network function provided by the sc_service crate with a BuildNetworkParams struct.
	// The build_network function returns a tuple containing the network object, which
	// represents the P2P network, the system_rpc_tx object, which is used for sending
	// system RPC messages, the tx_handler_controller, which is used to control the transaction
	// handler, and the network_starter, which is a future that starts the network.
	let (network, system_rpc_tx, tx_handler_controller, network_starter, syncing_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			// The block_announce_validator_builder field is an optional builder for a
			// validator that is used to validate incoming block announcements. If not
			// specified, the default validator will be used.
			block_announce_validator_builder: None,
			// The warp_sync field is an optional parameter for the network provider, which
			// provides the ability to do fast syncing of the blockchain using GRANDPA finality proofs.
			warp_sync_params: Some(WarpSyncParams::WithProvider(warp_sync)),
		})?;

	// offchain workers are used to perform tasks off the blockchain network. They can be used
	// for tasks such as fetching data from external sources, performing computations that
	// do not need to be stored on the blockchain, and sending transactions.
	//
	// Offchain workers are executed by a node's offchain worker pool, which is responsible
	// for managing the workers and executing their tasks. Each offchain worker has access
	// to the node's local state and can use it to execute its task.
	//
	// Some common use cases for offchain workers include:
	//
	// Fetching price data from external sources
	// Performing computations on behalf of the node, such as encryption or decryption
	// Submitting transactions to the network on behalf of the node
	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	// specifies the role of the node, which can be one of "FULL", "LIGHT", "AUTHORITY", or "INFRASTRUCTURE".
	let role = config.role.clone();
	// specifies whether the node should force block authoring, which means that it will
	// always try to produce blocks even if it is not an authority node.
	let force_authoring = config.force_authoring;
	// specifies the number of blocks the node should wait before attempting to produce
	// another block, if it is an authority node.
	let backoff_authoring_blocks: Option<()> = None;
	// specifies the name of the node.
	let name = config.network.node_name.clone();
	// specifies whether Grandpa finality should be enabled.
	let enable_grandpa = !config.disable_grandpa;
	// specifies the Prometheus registry to use for metrics, if any.
	let prometheus_registry = config.prometheus_registry().cloned();

	// The closure is used to create a full RPC extension that includes all the necessary
	// dependencies. The dependencies include a reference to the client and transaction pool,
	// which are needed to handle RPC requests. The deny_unsafe argument is used to specify
	// whether unsafe RPC methods should be denied.
	//
	// The rpc_extensions_builder is later used by the node to create and expose a set of
	// custom RPC methods to clients that connect to the node.
	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();

		Box::new(move |deny_unsafe, _| {
			let deps =
				crate::rpc::FullDeps { client: client.clone(), pool: pool.clone(), deny_unsafe };
			crate::rpc::create_full(deps).map_err(Into::into)
		})
	};

	// The spawn_tasks function in Substrate is responsible for spawning various background
	// tasks that are necessary for the functioning of the node. These tasks include:
	//
	// Consensus engine: This is the main task responsible for validating and importing new blocks as they arrive.
	// Transaction pool: This task is responsible for managing the pending transactions in the node's mempool.
	// RPC server: This task runs the node's JSON-RPC API, which allows external clients to interact with the node.
	// Keystore: This task is responsible for managing the node's key storage and signing transactions.
	// Telemetry: This task reports telemetry data about the node's performance back to the Substrate network.
	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network: network.clone(),
		client: client.clone(),
		keystore: keystore_container.sync_keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		// A closure that builds the RPC extensions for the node.
		rpc_builder: rpc_extensions_builder,
		backend,
		// The transaction sender for the system.
		system_rpc_tx,
		// The transaction handler controller.
		tx_handler_controller,
		config,
		telemetry: telemetry.as_mut(),
		sync_service: syncing_service
	})?;

	if role.is_authority() {
		// the code initializes a ProposerFactory using sc_basic_authorship::ProposerFactory::new()
		// function. The ProposerFactory is used to create a Proposer instance that is responsible
		// for proposing new blocks to the network.
		let proposer_factory = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

		// This code starts the consensus engine for the Aura algorithm, which is used to produce
		// blocks in a proof-of-authority (PoA) consensus network in Substrate.
		//
		// The function sc_consensus_aura::start_aura is called with various parameters,
		// including slot_duration which defines the length of a single slot in the network,
		// client which is the Substrate client, select_chain which selects the best chain to
		// build on, block_import which imports and validates new blocks, and proposer_factory
		// which creates instances of block proposers.
		//
		// The function also takes a closure that creates inherent data providers, which are used
		// to provide some data that is required for a block to be valid. In this case, the
		// closure creates an instance of sp_timestamp::InherentDataProvider and an instance of
		// sp_consensus_aura::inherents::InherentDataProvider based on the current system time
		// and the slot duration.
		//
		// Other parameters include force_authoring and backoff_authoring_blocks which control
		// how often blocks are authored, keystore_container which holds the keys required for
		// block authoring, and network which is the Substrate network protocol instance.
		//
		// The resulting aura object is the main entry point to the Aura consensus engine,
		// and will be used to produce new blocks in the network.
		let aura = sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _, _>(
			StartAuraParams {
				slot_duration,
				client,
				select_chain,
				block_import,
				proposer_factory,
				create_inherent_data_providers: move |_, ()| async move {
					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

					let slot =
						sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);

					Ok((slot, timestamp))
				},
				force_authoring,
				backoff_authoring_blocks,
				keystore: keystore_container.sync_keystore(),
				sync_oracle: syncing_service.clone(),
				justification_sync_link: syncing_service.clone(),
				block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
				max_block_proposal_slot_portion: None,
				telemetry: telemetry.as_ref().map(|x| x.handle()),
				compatibility_mode: Default::default(),
			},
		)?;

		// the AURA authoring task is considered essential, i.e. if it
		// fails we take down the service with it.

		// This code spawns a new blocking task using the task_manager to run the Aura
		// block authoring algorithm.
		//
		// The aura variable is a result of calling the sc_consensus_aura::start_aura()
		// function which creates an instance of the Aura block authoring algorithm with the given parameters.
		//
		// The spawn_blocking() method is called on the task_manager to spawn the aura
		// task in a new blocking thread. The first argument is a string identifier for the
		// task, while the second argument is an optional string to give further context to the task.
		//
		// Overall, this code sets up and starts the Aura block authoring algorithm in a
		// new thread managed by the task_manager.
		task_manager
			.spawn_essential_handle()
			.spawn_blocking("aura", Some("block-authoring"), aura);
	}

	if enable_grandpa {
		// if the node isn't actively participating in consensus then it doesn't
		// need a keystore, regardless of which protocol we use below.
		let keystore =
			if role.is_authority() { Some(keystore_container.sync_keystore()) } else { None };

		let grandpa_config = sc_consensus_grandpa::Config {
			// FIXME #1578 make this available through chainspec
			gossip_duration: Duration::from_millis(333),
			justification_period: 512,
			name: Some(name),
			observer_enabled: false,
			keystore,
			local_role: role,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			protocol_name: grandpa_protocol_name,
		};

		// start the full GRANDPA voter
		// NOTE: non-authorities could run the GRANDPA observer protocol, but at
		// this point the full voter should provide better guarantees of block
		// and vote data availability than the observer. The observer has not
		// been tested extensively yet and having most nodes in a network run it
		// could lead to finality stalls.
		let grandpa_config = sc_consensus_grandpa::GrandpaParams {
			config: grandpa_config,
			link: grandpa_link,
			network,
			voting_rule: sc_consensus_grandpa::VotingRulesBuilder::default().build(),
			prometheus_registry,
			shared_voter_state: SharedVoterState::empty(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
		};

		// the GRANDPA voter task is considered infallible, i.e.
		// if it fails we take down the service with it.
		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			None,
			sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
		);
	}

	network_starter.start_network();
	Ok(task_manager)
}
