// Descriptor wallet library extending bitcoin & miniscript functionality
// by LNP/BP Association (https://lnp-bp.org)
// Written in 2020-2022 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the Apache-2.0 License
// along with this software.
// If not, see <https://opensource.org/licenses/Apache-2.0>.

use std::collections::BTreeMap;

use bitcoin::hashes::{hash160, ripemd160, sha256, sha256d};
use bitcoin::psbt::PsbtSigHashType;
use bitcoin::util::bip32::KeySource;
use bitcoin::util::taproot::{ControlBlock, LeafVersion, TapBranchHash, TapLeafHash};
use bitcoin::{
    secp256k1, EcdsaSig, OutPoint, PublicKey, SchnorrSig, Script, Transaction, TxIn, TxOut,
    Witness, XOnlyPublicKey,
};

use crate::raw;
use crate::v0::InputV0;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Input {
    /// Previous transaction outpoint to spent.
    pub previous_outpoint: OutPoint,

    /// Sequence number of this input. If omitted, the sequence number is
    /// assumed to be the final sequence number (0xffffffff).
    pub sequence_number: Option<u32>,

    /// 32 bit unsigned little endian integer greater than or equal to 500000000
    /// representing the minimum Unix timestamp that this input requires to be
    /// set as the transaction's lock time.
    required_time_locktime: Option<u32>,

    /// 32 bit unsigned little endian integer less than 500000000 representing
    /// the minimum block height that this input requires to be set as the
    /// transaction's lock time.
    required_height_locktime: Option<u32>,

    /// The non-witness transaction this input spends from. Should only be
    /// `Some` for inputs which spend non-segwit outputs or if it is unknown
    /// whether an input spends a segwit output.
    pub non_witness_utxo: Option<Transaction>,

    /// The transaction output this input spends from. Should only be `Some` for
    /// inputs which spend segwit outputs, including P2SH embedded ones.
    pub witness_utxo: Option<TxOut>,

    /// A map from public keys to their corresponding signature as would be
    /// pushed to the stack from a scriptSig or witness for a non-taproot
    /// inputs.
    pub partial_sigs: BTreeMap<PublicKey, EcdsaSig>,

    /// The sighash type to be used for this input. Signatures for this input
    /// must use the sighash type.
    pub sighash_type: Option<PsbtSigHashType>,

    /// The redeem script for this input.
    pub redeem_script: Option<Script>,

    /// The witness script for this input.
    pub witness_script: Option<Script>,

    /// A map from public keys needed to sign this input to their corresponding
    /// master key fingerprints and derivation paths.
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::btreemap_as_seq"))]
    pub bip32_derivation: BTreeMap<secp256k1::PublicKey, KeySource>,

    /// The finalized, fully-constructed scriptSig with signatures and any other
    /// scripts necessary for this input to pass validation.
    pub final_script_sig: Option<Script>,

    /// The finalized, fully-constructed scriptWitness with signatures and any
    /// other scripts necessary for this input to pass validation.
    pub final_script_witness: Option<Witness>,

    /// TODO: Proof of reserves commitment

    /// RIPEMD160 hash to preimage map.
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::btreemap_byte_values"))]
    pub ripemd160_preimages: BTreeMap<ripemd160::Hash, Vec<u8>>,

    /// SHA256 hash to preimage map.
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::btreemap_byte_values"))]
    pub sha256_preimages: BTreeMap<sha256::Hash, Vec<u8>>,

    /// HSAH160 hash to preimage map.
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::btreemap_byte_values"))]
    pub hash160_preimages: BTreeMap<hash160::Hash, Vec<u8>>,

    /// HAS256 hash to preimage map.
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::btreemap_byte_values"))]
    pub hash256_preimages: BTreeMap<sha256d::Hash, Vec<u8>>,

    /// Serialized schnorr signature with sighash type for key spend.
    pub tap_key_sig: Option<SchnorrSig>,

    /// Map of <xonlypubkey>|<leafhash> with signature.
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::btreemap_as_seq"))]
    pub tap_script_sigs: BTreeMap<(XOnlyPublicKey, TapLeafHash), SchnorrSig>,

    /// Map of Control blocks to Script version pair.
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::btreemap_as_seq"))]
    pub tap_scripts: BTreeMap<ControlBlock, (Script, LeafVersion)>,

    /// Map of tap root x only keys to origin info and leaf hashes contained in
    /// it.
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::btreemap_as_seq"))]
    pub tap_key_origins: BTreeMap<XOnlyPublicKey, (Vec<TapLeafHash>, KeySource)>,

    /// Taproot Internal key.
    pub tap_internal_key: Option<XOnlyPublicKey>,

    /// Taproot Merkle root.
    pub tap_merkle_root: Option<TapBranchHash>,

    /// Proprietary key-value pairs for this input.
    #[cfg_attr(
        feature = "serde",
        serde(with = "::serde_utils::btreemap_as_seq_byte_values")
    )]
    pub proprietary: BTreeMap<raw::ProprietaryKey, Vec<u8>>,

    /// Unknown key-value pairs for this input.
    #[cfg_attr(
        feature = "serde",
        serde(with = "::serde_utils::btreemap_as_seq_byte_values")
    )]
    pub unknown: BTreeMap<raw::Key, Vec<u8>>,
}

impl Input {
    pub fn with(v0: InputV1, txin: TxIn) -> Self {
        let sequence = match txin.sequence {
            u32::MAX => None,
            other => Some(other),
        };

        Input {
            previous_outpoint: txin.previous_output,
            sequence_number: sequence,
            required_time_locktime: None,
            required_height_locktime: None,
            non_witness_utxo: v0.non_witness_utxo,
            witness_utxo: v0.witness_utxo,
            partial_sigs: v0.partial_sigs,
            sighash_type: v0.sighash_type,
            redeem_script: v0.redeem_script,
            witness_script: v0.witness_script,
            bip32_derivation: v0.bip32_derivation,
            final_script_sig: v0.final_script_sig,
            final_script_witness: v0.final_script_witness,
            ripemd160_preimages: v0.ripemd160_preimages,
            sha256_preimages: v0.sha256_preimages,
            hash160_preimages: v0.hash160_preimages,
            hash256_preimages: v0.hash256_preimages,
            tap_key_sig: v0.tap_key_sig,
            tap_script_sigs: v0.tap_script_sigs,
            tap_scripts: v0.tap_scripts,
            tap_key_origins: v0.tap_key_origins,
            tap_internal_key: v0.tap_internal_key,
            tap_merkle_root: v0.tap_merkle_root,
            proprietary: v0.proprietary,
            unknown: v0.unknown,
        }
    }

    #[inline]
    pub fn locktime(&self) -> Option<u32> {
        self.required_time_locktime
            .or_else(self.required_height_locktime)
    }

    pub fn split(self) -> (InputV0, TxIn) {
        (
            InputV0 {
                non_witness_utxo: self.non_witness_utxo,
                witness_utxo: self.witness_utxo,
                partial_sigs: self.partial_sigs,
                sighash_type: self.sighash_type,
                redeem_script: self.redeem_script,
                witness_script: self.witness_script,
                bip32_derivation: self.bip32_derivation,
                final_script_sig: self.final_script_sig,
                final_script_witness: self.final_script_witness,
                ripemd160_preimages: self.ripemd160_preimages,
                sha256_preimages: self.sha256_preimages,
                hash160_preimages: self.hash160_preimages,
                hash256_preimages: self.hash256_preimages,
                tap_key_sig: self.tap_key_sig,
                tap_script_sigs: self.tap_script_sigs,
                tap_scripts: self.tap_scripts,
                tap_key_origins: self.tap_key_origins,
                tap_internal_key: self.tap_internal_key,
                tap_merkle_root: self.tap_merkle_root,
                proprietary: self.proprietary,
                unknown: self.unknown,
            },
            TxIn {
                previous_output: self.previous_outpoint,
                script_sig: Default::default(),
                sequence: self.sequence_number.unwrap_or(u32::MAX),
                witness: Default::default(),
            },
        )
    }
}

impl From<Input> for InputV1 {
    fn from(input: Input) -> Self {
        InputV1 {
            non_witness_utxo: input.non_witness_utxo,
            witness_utxo: input.witness_utxo,
            partial_sigs: input.partial_sigs,
            sighash_type: input.sighash_type,
            redeem_script: input.redeem_script,
            witness_script: input.witness_script,
            bip32_derivation: input.bip32_derivation,
            final_script_sig: input.final_script_sig,
            final_script_witness: input.final_script_witness,
            ripemd160_preimages: input.ripemd160_preimages,
            sha256_preimages: input.sha256_preimages,
            hash160_preimages: input.hash160_preimages,
            hash256_preimages: input.hash256_preimages,
            tap_key_sig: input.tap_key_sig,
            tap_script_sigs: input.tap_script_sigs,
            tap_scripts: input.tap_scripts,
            tap_key_origins: input.tap_key_origins,
            tap_internal_key: input.tap_internal_key,
            tap_merkle_root: input.tap_merkle_root,
            proprietary: input.proprietary,
            unknown: input.unknown,
        }
    }
}
