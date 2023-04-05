use frame_support::{
    construct_runtime,
    parameter_types,
    traits::{
        ConstU64,
        ConstU32,
        ConstU128,
        /// Something which can compute and check proofs of a historical key owner and return full identification data of that key owner.
        KeyOwnerProofSystem,
    },
    weights::{
        constants::{
            WEIGHT_REF_TIME_PER_SECOND,
            /// By default, Substrate uses RocksDB, so this will be the weight used throughout the runtime.
            RocksDbWeight,
        },
        Weight,
    },
};
pub use sp_runtime::{
    /// Perbill - Re-export top-level arithmetic stuff. A fixed point representation of a number in the range [0, 1].
    Perbill,
    Permill,
    create_runtime_str,
    traits::{
        /// Blake2-256 Hash implementation.
        /// https://crates.parity.io/sp_runtime/traits/struct.BlakeTwo256.html
        BlakeTwo256,
        /// Some type that is able to be collapsed into an account ID. It is not possible to recreate the original value from the account ID.
        /// https://docs.rs/sp-runtime/latest/sp_runtime/traits/trait.IdentifyAccount.html
        IdentifyAccount,
        /// Means of signature verification. / Verify a signature.
        /// https://docs.rs/sp-runtime/latest/sp_runtime/traits/trait.Verify.html
        Verify,
        /// A lookup implementation returning the AccountId from a MultiAddress.
        /// https://docs.rs/sp-runtime/22.0.0/sp_runtime/traits/struct.AccountIdLookup.html
        AccountIdLookup,
    },
    MultiSignature,
    /// Generic implementations of Extrinsic/Header/Block.
    /// https://docs.rs/sp-runtime/22.0.0/sp_runtime/generic/index.html
    generic,
};

///Runtime version. This should not be thought of as classic Semver (major/minor/tiny). This triplet have
/// different semantics and mis-interpretation could cause problems. In particular:
/// bug fixes should result in an increment of spec_version and possibly authoring_version,
/// absolutely not impl_version since they change the semantics of the runtime.
use sp_version::RuntimeVersion;

/// An Aura authority identifier using S/R 25519 as its crypto.
///
/// type alias in the Substrate Rust implementation of the Aura consensus algorithm. It represents
/// the unique identifier of an authority node in the network, which is a public key generated
/// using the sr25519 signature algorithm.
use sp_consensus_aura::sr25519::AuthorityId as AuraId;

use sp_core::{
    /// is used to define a unique identifier for a specific cryptographic key type used in the
    /// substrate runtime. It is used to differentiate different types of keys that may be used
    /// in the runtime and to ensure that they are used correctly.
    /// For example, when defining a new module that requires cryptographic keys, a new KeyTypeId
    /// may be defined to represent that key type. This ensures that the keys are used correctly
    /// by the module and not confused with keys used by other parts of the runtime.
    crypto::KeyTypeId,
    OpaqueMetadata,
};

/// pallet for block finalization
use pallet_grandpa::{
    /// Primitives for GRANDPA integration, suitable for WASM compilation.
    /// Here are the primitives that you will need to define for Grandpa integration:

    /// GrandpaAuthorities: This is a type that represents the set of Grandpa authorities for a given era.
    /// 					It should contain a vector of AuthorityId objects, which represent the public keys of the authorized validators.

    /// GrandpaId: This is a type that represents the identifier for a given era in Grandpa.
    /// 		   It is typically a hash of the set of authorities for that era.

    /// GrandpaJustification: This is a type that represents the justification for a given block in Grandpa.
    /// 					  It should contain a vector of signed messages that attest to the validity of the block.

    /// GrandpaPrevote: This is a type that represents a prevote message in Grandpa.
    /// 				It contains the identifier for the era being voted on, as well as a hash of the block being voted for.

    /// GrandpaPrecommit: This is a type that represents a precommit message in Grandpa.
    /// 				  It contains the same information as a prevote, as well as a justification for the block being voted for.
    fg_primitives,
    /// Identity of a Grandpa authority.
    /// sp_runtime::MultiAddress<sp_runtime::MultiSigner, ()>, which is a type that represents a Polkadot
    /// account that is authorized to act as a validator in the Grandpa consensus algorithm.
    /// use sp_runtime::MultiAddress;
    /// use sp_runtime::traits::IdentifyAccount;
    /// use sp_core::sr25519::Public;
    /// use sp_runtime::MultiSigner;

    /// let public_key = Public::from_raw(*b"12345678901234567890123456789012");
    /// let account_id = <MultiSigner as IdentifyAccount>::new(public_key.into());
    /// let authority_id = MultiAddress::Id(account_id);
    /// In this example, public_key is a 32-byte raw representation of the public key of the validator.
    AuthorityId as GrandpaId,
    /// is a type that represents the list of Grandpa authorities for a given era in the Polkadot network.
    /// It is typically stored in the runtime's storage and updated at the end of each era to reflect changes in the set of active validators.
    /// use pallet_grandpa::{AuthorityList, AuthorityId};
    /// use sp_core::sr25519::Public;
    /// use sp_runtime::traits::IdentifyAccount;
    /// use sp_runtime::MultiSigner;
    /// use sp_std::vec::Vec;

    /// let authority_1_key = Public::from_raw(*b"12345678901234567890123456789012");
    /// let authority_1_account_id = <MultiSigner as IdentifyAccount>::new(authority_1_key.into());
    /// let authority_1_id = AuthorityId::from(authority_1_account_id);

    /// let authority_2_key = Public::from_raw(*b"12345678901234567890123456789013");
    /// let authority_2_account_id = <MultiSigner as IdentifyAccount>::new(authority_2_key.into());
    /// let authority_2_id = AuthorityId::from(authority_2_account_id);

    /// let authority_list = AuthorityList {
    /// 	authorities: vec![authority_1_id, authority_2_id],
    /// 	last_block_number: 12345,
    ///  };
    AuthorityList as GrandpaAuthorityList, /// A list of Grandpa authorities with associated weights.
};

/// Converts a percent into Self. Equal to x / 100.
/// This can be created at compile time.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// An index to a block.
pub type BlockNumber = u32;
pub type Index = u32;
/// A hash of some data used by the chain.
/// Fixed-size uninterpreted hash type with 32 bytes (256 bits) size.
/// pub struct H256(pub [u8; 32]);
/// https://docs.rs/sp-core/latest/sp_core/struct.H256.html
pub type Hash = sp_core::H256;
/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
/// pub enum MultiSignature {
/// 	Ed25519(Signature),
/// 	Sr25519(Signature),
/// 	Ecdsa(Signature),
/// }
/// By using MultiSignature as the base type for Signature, the runtime allows for the use of multiple signature schemes 
/// within a single blockchain network.
/// In Substrate, signatures are used for transaction authorization and validation. When a transaction is created, 
/// it is signed using the private key of the account that initiates the transaction. This signature is then included 
/// with the transaction when it is submitted to the network. When a node receives a transaction, it verifies the 
/// signature to ensure that the transaction was indeed initiated by the account that claims to have initiated it.
/// By using MultiSignature, the runtime provides flexibility in the types of cryptographic signatures that can be 
/// used in a blockchain network, making it possible to support multiple signature schemes and accommodate different use cases.
/// https://docs.rs/sp-runtime/latest/sp_runtime/enum.MultiSignature.html
pub type Signature = MultiSignature;
/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
/// The account ID that this can be transformed into.
/// https://docs.rs/sp-runtime/latest/sp_runtime/traits/trait.IdentifyAccount.html#associatedtype.AccountId
/// 1. The Verify trait is implemented for a given signature algorithm (e.g. sr25519, ed25519) which provides 
/// a method to verify the authenticity of a signature.
/// 2. The Signer trait is implemented for the verified signature, which provides a method to identify the 
/// account that created the signature.
/// 3. The IdentifyAccount trait is implemented for the signer identity, which provides a method to retrieve 
/// the account identifier for the signer.
/// 4. Finally, the AccountId type is defined as the account identifier for the signer identity.
/// Therefore, the AccountId type is a composite of the signer identity and the signature algorithm used to 
/// sign the transaction. In the context of the Substrate runtime, this type is used to represent the unique 
/// identifier of an account that participates in the blockchain network. This identifier is used for various 
/// purposes such as transaction authorization, balance tracking, and staking.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
/// Balance of an account.
pub type Balance = u128;

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
    pub const SS58Prefix: u8 = 42;
    pub const BlockHashCount: BlockNumber = 2400;
	/// Using const to create a parameter type that provides a const getter. It is required that the value is const.
    /// Declare the parameter type without const to have more freedom when creating the value.

	/// We allow for 2 seconds of compute with a 6 second average block time.
    /// using in frame_system
    /// `limit` - Block resource limits configuration structures. Weight (execution cost/time) Length (block size)
    /// `BlockWeights` - Block weight limits & base values configuration.
    /// Create a sensible default weights system given only expected maximal block weight and the ratio that Normal extrinsics should occupy.
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::with_sensible_defaults(
			Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
			NORMAL_DISPATCH_RATIO,
		);

	/// Create new BlockLength with max(0.65536MB) for Operational & Mandatory and normal(0.75) * max(0.65536MB) for Normal.
	pub BlockLength: frame_system::limits::BlockLength =
		frame_system::limits::BlockLength::max_with_normal_ratio(
			5*1024*1024, NORMAL_DISPATCH_RATIO
		);
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
/// An attribute that accepts a version declaration of a runtime and generates a custom wasm section with the equivalent contents.
/// The custom section allows to read the version of the runtime without having to execute any code. Instead,
/// the generated custom section can be relatively easily parsed from the wasm binary. The identifier of the custom section is “runtime_version”.
/// It will pass it through and add code required for emitting a custom section. The information that will
/// go into the custom section is parsed from the item declaration. Due to that, the macro is somewhat rigid in terms of the code it accepts.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    /// Identifies the different Substrate runtimes. There’ll be at least polkadot and node. A different on-chain
    /// spec_name to that of the native runtime would normally result in node not attempting to sync or author blocks.
    ///
    /// The spec_name field identifies the runtime that the node is running. It is used to distinguish between different
    /// runtime versions and is set during the runtime construction process using create_runtime_str!() macro.
    /// In practice, a runtime's spec_name should remain consistent throughout its lifetime, and the node will only
    /// try to sync and author blocks if the on-chain spec_name matches the node's spec_name.
    ///
    /// When a node attempts to sync or author blocks, it checks the spec_name value of the native runtime against
    /// that of the on-chain runtime. If the spec_name values are different, the node will not attempt to use its
    /// native runtime in substitute for the on-chain Wasm runtime. This helps ensure that nodes are always working
    /// with the correct runtime and prevent any potential chain splits.
    spec_name: create_runtime_str!("simple-commented-node-template"),
    /// The impl_name field in the RuntimeVersion struct specifies the name of the implementation of the runtime.
    /// This is useful when there are multiple implementations of a runtime and it helps to distinguish them from one another.
    /// For example, in the Polkadot network, there are multiple implementations of the Polkadot runtime,
    /// including substrate, parity, and gossamer. Each of these implementations may have slight differences
    /// in how they are built, optimized, or deployed, and specifying the impl_name helps to distinguish them from one another.
    /// Additionally, having a well-defined implementation name makes it easier to write client software that
    /// is compatible with multiple implementations of the same runtime, as the implementation name can be used
    /// as a key to detect differences in behavior or features.
    ///
    /// If spec_name and impl_name are different, it means that the implementation of the runtime is different from
    /// its on-chain specification. This could cause issues if the on-chain specification and the implementation
    /// do not agree on certain rules or behaviors. For example, nodes may not be able to sync or author blocks
    /// if the spec_name of the runtime does not match that of the on-chain specification. Therefore, it is
    /// generally recommended to keep the spec_name and impl_name the same to ensure consistency between the
    /// on-chain specification and the implementation.
    impl_name: create_runtime_str!("simple-commented-node-template"),
    /// used to specify the version of the runtime that is used to author blocks. It is an integer value
    /// that should be incremented whenever the runtime code is changed in a way that affects the format or
    /// validity of the block data that is produced by the runtime.
    /// By incrementing the authoring_version value, nodes can signal to other nodes in the network that
    /// they have updated their runtime code and are producing blocks using the latest version of the runtime.
    /// This allows nodes to stay in sync with each other and ensures that the network operates smoothly.
    authoring_version: 1,
    // The version of the runtime specification. A full node will not attempt to use its native
    //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    //   `spec_version`, and `authoring_version` are the same between Wasm and native.
    // This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
    //   the compatible custom types.
    /// used to specify the version of the runtime specification that the runtime is implementing.
    /// It is an integer value that increases whenever there is a breaking change to the runtime specification.
    /// This value is used to ensure that nodes using different versions of the runtime specification cannot
    /// interoperate and that nodes running the same version of the runtime specification can interoperate.
    /// For example, if a breaking change is made to the runtime specification, nodes that have not yet
    /// updated to the new version will not be able to validate blocks produced by nodes that have
    /// already updated to the new version. By requiring nodes to use the same spec_version, the network
    /// is able to ensure that all nodes are using the same version of the runtime specification and that
    /// they can all validate the same set of blocks.
    spec_version: 100,
    /// specifies the version number of the runtime implementation. It is used to identify different
    /// implementations of the same runtime specification. For example, if there are two implementations
    /// of the same runtime specification, each with its own unique features, they would have different impl_version values.
    impl_version: 1,
    /// is a field that contains the list of supported runtime API versions.
    /// In Substrate, runtime APIs are versioned independently from the runtime specification,
    /// which means that they can be updated or extended without changing the runtime specification or
    /// breaking backwards compatibility. The RUNTIME_API_VERSIONS constant is typically defined in the
    /// runtime and lists the API versions that the runtime supports.
    apis: RUNTIME_API_VERSIONS,
    /// used to track the current version of the extrinsic transaction format used by the runtime.
    /// When a new extrinsic format is introduced or changes to the existing format are made, this
    /// version number is incremented. This ensures that the correct transaction format is used when
    /// sending transactions to the runtime, and also helps to prevent backward compatibility issues.
    ///
    /// This number must change when an existing dispatchable (module ID, dispatch ID) is changed, either
    /// through an alteration in its user-level semantics, a parameter added/removed/changed, a
    /// dispatchable being removed, a module being removed, or a dispatchable/module changing its index.
    ///
    /// It need not change when a new module is added or when a dispatchable is added.
    transaction_version: 1,
    /// used to indicate changes in the storage format of the runtime. Whenever there is a change
    /// in the storage format, the state_version should be incremented.
    /// Use of an incorrect version is consensus breaking.
    state_version: 1,
};

impl frame_system::Config for Runtime {
    /// The CallFilter trait is used to determine which extrinsics are allowed to be included in a block.
    /// Therefore, BaseCallFilter: Contains<Self::RuntimeCall> means that the filter will only allow extrinsics
    /// containing a specific RuntimeCall type to be included in a block. This allows for more fine-grained
    /// control over the types of transactions that can be included in the blockchain.
    ///
    /// In Substrate, the frame_support::traits::Everything type is used as a call filter to allow
    /// all extrinsics to be included in a block. This means that any extrinsic, regardless of its Call
    /// type, will be accepted by the node and included in the blockchain.
    type BaseCallFilter = frame_support::traits::Everything;

    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = BlockWeights;

    /// The maximum length of a block (in bytes).
    type BlockLength = BlockLength;

    /// The RuntimeOrigin type used by dispatchable calls.
    /// The runtime origin is used by dispatchable functions to check where a call has come from.
    /// The RuntimeOrigin trait is used extensively throughout the Substrate runtime to enforce
    /// permission checks and access control. It provides a flexible mechanism for defining custom
    /// origin types that can be used to restrict access to certain functions or actions within the runtime.
    ///
    /// WHO CALL
    type RuntimeOrigin = RuntimeOrigin;

    /// The aggregated dispatch type that is available for extrinsics.
    /// The RuntimeCall enum is used in the Substrate runtime to enforce access control based on the origin of a particular call.
    /// When a function or module is called within the runtime, the origin of the call is passed along with it.
    /// The origin identifies the source of the call, such as a specific account or the runtime itself.
    /// The RuntimeCall associated type of the RuntimeOrigin trait is used to ensure that only authorized
    /// origins are allowed to make certain calls. For example, imagine a runtime that has two functions:
    /// transfer() and mint(). The transfer() function should only be callable by an authorized account,
    /// while the mint() function should only be callable by the runtime itself. To enforce this access control,
    /// the RuntimeCall associated type is used to associate the transfer() function with the Transfer variant of
    /// the RuntimeCall enum, and the mint() function with a different variant of the RuntimeCall enum. Then,
    /// the RuntimeOrigin trait can be implemented to check the origin of a call against a whitelist of authorized
    /// origins for each variant of the RuntimeCall enum.
    ///
    /// WHAT CAN CALL
    type RuntimeCall = RuntimeCall;

    /// Account index (aka nonce) type. This stores the number of previous transactions associated with a sender account.
    /// Or the index type for storing how many extrinsics an account has signed.
    type Index = Index;

    /// The block number type used by the runtime.
    type BlockNumber = BlockNumber;

    /// The type for hashing blocks and tries./ The output of the Hashing function.
    type Hash = Hash;

    /// The hashing system (algorithm) being used in the runtime (e.g. Blake2).
    /// By defining a specific hashing algorithm in the runtime, the blockchain can ensure that all
    /// nodes in the network use the same algorithm to perform these important operations. This helps
    /// to ensure consistency and security across the network, and reduces the risk of attacks or data tampering.
    ///
    /// 1. Block and transaction validation: Each block and transaction in the runtime is hashed using the By defining a specific
    /// 	hashing algorithm in the runtime, the blockchain can ensure that all nodes in the network use the same algorithm
    /// 	to perform these important operations. This helps to ensure consistency and security across the network, and reduces
    /// 	the risk of attacks or data tampering. specified hashing algorithm. This hash is then used to validate the integrity of the data in
    /// 	the block or transaction. If the hash does not match the expected value, the block or transaction is considered invalid.
    /// 2. State storage: The runtime uses the hashing algorithm to generate a unique key for each piece of
    /// 	data that is stored in the state. This allows for efficient storage and retrieval of data, as well as
    /// 	providing a tamper-evident way to verify that the data has not been modified.
    /// 3. Consensus mechanism: Many consensus algorithms used in blockchain networks rely on hash functions to
    /// 	produce a deterministic output that can be used to select a leader or validator for the next block.
    ///
    /// Substrate supports several hashing algorithms that can be used in a runtime. These include:
    /// 1. Blake2b: A fast and secure cryptographic hash function that produces a 256-bit output.
    /// 2. Sha2_256: A variant of the SHA-2 family of hash functions that produces a 256-bit output.
    /// 3. Keccak256: A variant of the Keccak family of hash functions that produces a 256-bit output.
    /// 4. Sha3_256: A variant of the SHA-3 family of hash functions that produces a 256-bit output.
    /// 5. Twox64Concat: A custom hash function used in the Polkadot runtime that concatenates two 64-bit inputs
    /// 	and hashes them using the Blake2b algorithm.
    type Hashing = BlakeTwo256;

    /// The user account identifier type for the runtime.
    type AccountId = AccountId;

    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    ///
    /// In summary, the Lookup type is a convenient way to perform lookups on account identifiers (AccountId)
    /// and is used to retrieve information associated with an account. In this case, the () parameter is
    /// used because the lookup function does not return any additional data for the specified account.
    ///
    /// Converting trait to take a source type and convert to AccountId.
    /// Used to define the type and conversion mechanism for referencing accounts in transactions.
    /// It’s perfectly reasonable for this to be an identity conversion (with the source type being AccountId),
    /// but other pallets (e.g. Indices pallet) may provide more functional/efficient alternatives.
    type Lookup = AccountIdLookup<AccountId, ()>;

    /// Abstraction over a block header for a substrate chain.
    /// pub struct Header<Number: Copy + Into<U256> + TryFrom<U256>, Hash: HashT> {
    ///    	pub parent_hash: Hash::Output,
    ///    	pub number: Number,
    ///    	pub state_root: Hash::Output,
    ///    	pub extrinsics_root: Hash::Output,
    ///    	pub digest: Digest,
    ///    }
    /// It represents a block header in the Substrate blockchain and is parameterized by the BlockNumber and BlakeTwo256 types.
    /// The BlockNumber type specifies the number of blocks in the blockchain and is typically implemented as a 64-bit unsigned integer.
    /// The BlakeTwo256 type is the hashing algorithm used to hash the block header. It is a variant of the Blake2b hash
    /// function that produces a 256-bit hash value.
    /// Together, the Header type represents the metadata of a block in the Substrate blockchain, including its block number and hash value.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// TODO
    type RuntimeEvent = RuntimeEvent;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;

    /// The weight of runtime database operations the runtime can invoke.
    /// The RocksDbWeight is a specific implementation of the RuntimeDbWeight struct that is used in Substrate's RocksDB storage backend.
    /// The weight is measured in abstract units rather than bytes. It represents the amount of resources required to
    /// execute a particular operation on the database. (read/write)
    type DbWeight = RocksDbWeight;

    /// Get the chain’s current version.
    type Version = Version;

    /// Expects the PalletInfo type that is being generated by construct_runtime! in the runtime.
    ///
    /// PalletInfo trait that provides information about a pallet (a module) in the runtime. It includes the
    /// name of the pallet, the author, the version, and other metadata. The PalletInfo trait can
    /// be implemented by pallets and used by other pallets or the runtime to retrieve information about the pallets.
    type PalletInfo = PalletInfo;

    /// Data to be associated with an account (other than nonce/transaction counter, which this pallet does regardless).
    ///
    /// The AccountData type is defined in the pallet_balances pallet of a Substrate runtime. It is a struct that contains two fields:
    /// free: A Balance type representing the free balance that can be transferred or used in transactions.
    /// reserved: A Balance type representing the balance that has been reserved for some specific use, such as for a staking or a vesting system.
    type AccountData = pallet_balances::AccountData<Balance>;

    /// Handler for when a new account has just been created.
    type OnNewAccount = ();

    /// A function that is invoked when an account has been determined to be dead.
    /// All resources should be cleaned up associated with the given account.
    type OnKilledAccount = ();

    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();

    /// it represents the prefix used to encode and decode addresses in the Substrate ecosystem.
    /// The prefix is used to identify the network to which an address belongs and allows different
    /// Substrate networks to coexist without conflicts.
    /// By default, Substrate networks use a unique SS58Prefix value to distinguish their network
    /// from other networks. However, in some cases, such as when creating a local development
    /// network, it may be necessary to use a different prefix. The SS58Prefix type provides a
    /// way to customize this behavior.
    type SS58Prefix = SS58Prefix;

    /// The purpose of this type is to represent a callback function that gets executed whenever the
    /// runtime code of a pallet is updated or replaced. In the Substrate framework, runtime code is
    /// executed by a WebAssembly (Wasm) binary that gets stored on-chain as part of the runtime state.
    //
    /// When the Wasm binary for a pallet is updated, Substrate executes this callback to perform any
    /// necessary actions to ensure that the new code is properly integrated with the existing runtime
    /// state. In this case, however, the tuple containing the empty unit value indicates that no
    /// such actions are necessary for this pallet.
    type OnSetCode = ();

    /// This type alias is used in the context of defining the maximum number of "consumers" that a
    /// certain component of a substrate node can support. In the substrate codebase, a "consumer"
    /// refers to a type of object that is capable of receiving input from another object (usually a runtime module).
    //
    /// For example, the sp_runtime::traits::UnfilteredDispatchable trait has a dispatch method that
    /// takes a frame_system::RawOrigin and an Vec<MaxEncodedLen> as input, where MaxEncodedLen is
    /// a wrapper around a Vec<u8> that implements the Encode and Decode traits. The Vec<MaxEncodedLen>
    /// is a list of arguments that are passed to the dispatch method. The MaxConsumers type alias
    /// is used to define the maximum number of arguments that can be passed to this method.
    type MaxConsumers = ConstU32<16>;
}

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch. current timestamp
    type Moment = u64;
    /// what should be notified when timestamp is set, can be ()
    type OnTimestampSet = Aura;
    /// time beetwen blocks
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    /// setup weight for set()
    type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
    /// It represents the type of the authority identifier used in a Substrate consensus algorithm,
    /// and it is used to specify the type of the authority that is used for a particular Substrate runtime.
    /// In the default Substrate runtime, the AuthorityId type is an alias for the AuraId type,
    /// which represents the identifier used in the Aura consensus algorithm, which is the default
    /// consensus algorithm for Substrate. Other consensus algorithms, such as Babe or Grandpa,
    /// have their own types of authority identifiers, and can be used by substituting them for the AuthorityId type.
    type AuthorityId = AuraId;
    /// it represents the maximum number of authorities that are allowed to participate in
    /// block production or other network activities.
    type MaxAuthorities = ConstU32<32>;
    /// A way to check whether a given validator is disabled and should not be authoring
    /// blocks. Blocks authored by a disabled validator will lead to a panic as part of this module’s initialization.
    type DisabledValidators = ();
}

impl pallet_grandpa::Config for Runtime {
    /// TODO
    type RuntimeEvent = RuntimeEvent;

    /// The KeyOwnerProofSystem is a trait that defines the behavior of a key ownership proof
    /// system, which is used to validate the authority set in the grandpa finality gadget. In
    /// the default substrate configuration, the KeyOwnerProofSystem is implemented by the
    /// sp_session::OpaqueKeys type, which is a type alias for OpaqueMultiPhase.
    /// If you want to customize the way key ownership proofs are handled, you can define a
    /// custom type that implements the KeyOwnerProofSystem trait and use it in your runtime.
    type KeyOwnerProofSystem = ();
    /// In this case, the KeyOwnerProof type is defined in the context of a runtime that has a
    /// KeyOwnerProofSystem implementation for the tuple (KeyTypeId, GrandpaId). This trait
    /// provides a way to verify ownership of keys
    type KeyOwnerProof =
    <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

    /// The identification of a key owner, used when reporting equivocations.
    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        GrandpaId,
    )>>::IdentificationTuple;

    /// The equivocation handling subsystem, defines methods to report an
    /// offence (after the equivocation has been validated) and for submitting a
    /// transaction to report an equivocation (from an offchain context).
    /// NOTE: when enabling equivocation handling (i.e. this type isn't set to
    /// `()`) you must use this pallet's `ValidateUnsigned` in the runtime
    /// definition.
    type HandleEquivocation = ();

    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
}

pub const EXISTENTIAL_DEPOSIT: u128 = 500;

impl pallet_balances::Config for Runtime {
    /// TODO
    type RuntimeEvent = ();

    /// The balance of an account.
    type Balance = Balance;
    /// is a placeholder type that represents the configuration for removing so-called "dust"
    /// accounts. Dust accounts are accounts that hold tiny amounts of a token, typically less
    /// than the minimum transaction fee, and can cause unnecessary bloat on the blockchain.
    /// In the Substrate runtime, dust accounts can be removed periodically to free up space on
    /// the blockchain. The DustRemoval type provides a way to configure this behavior, but in
    /// this case, it is set to the unit type (), which indicates that dust removal is disabled.
    type DustRemoval = ();

    /// This code defines a type alias named ExistentialDeposit which is equal to the constant
    /// unsigned 128-bit integer value of EXISTENTIAL_DEPOSIT. EXISTENTIAL_DEPOSIT is a constant
    /// that represents the minimum amount of funds required to be held in an account.
    ///
    /// In the substrate runtime, it is used as a mechanism to keep the blockchain state size
    /// manageable by allowing the removal of small or empty accounts that do not have a balance
    /// higher than EXISTENTIAL_DEPOSIT. When an account balance falls below the EXISTENTIAL_DEPOSIT,
    /// the account is removed from the state during the next pruning process.
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = ();
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
}

construct_runtime!(
	pub struct Runtime
	where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Aura: pallet_aura,
		Grandpa: pallet_grandpa,
		Balances: pallet_balances,
		TransactionPayment: pallet_transaction_payment,
		Sudo: pallet_sudo
	}
);