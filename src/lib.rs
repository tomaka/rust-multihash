/// ! # multihash
/// !
/// ! Implementation of [multihash](https://github.com/multiformats/multihash)
/// ! in Rust.
/// Representation of a Multiaddr.

#[cfg(feature = "encode")]
extern crate ring;
#[cfg(feature = "encode")]
extern crate tiny_keccak;

use std::fmt::Write;

#[cfg(feature = "encode")]
use tiny_keccak::Keccak;
#[cfg(feature = "encode")]
use ring::digest;

mod hashes;
pub use hashes::*;

mod errors;
pub use errors::*;

// Helper macro for encoding input into output using either ring or tiny_keccak
#[cfg(feature = "encode")]
macro_rules! encode {
    (ring, $algorithm:ident, $input:expr, $output:expr) => ({
        let result = digest::digest(&digest::$algorithm, $input);
        debug_assert!($output.len() == result.as_ref().len());
        $output.copy_from_slice(result.as_ref());
    });
    (tiny, $constructor:ident, $input:expr, $output:expr) => ({
        let mut kec = Keccak::$constructor();
        kec.update($input);
        kec.finalize($output);
    })
}

// And another one to keep the matching DRY
#[cfg(feature = "encode")]
macro_rules! match_encoder {
    ($hash:ident for ($input:expr, $output:expr) {
        $( $hashtype:ident => $lib:ident :: $method:ident, )*
    }) => ({
        match $hash {
            $(
                Hash::$hashtype => encode!($lib, $method, $input, $output),
            )*

            _ => return Err(Error::UnsupportedType)
        }
    })
}


/// Encodes data into a multihash.
///
/// The returned data is raw bytes.  To make is more human-friendly, you can encode it (hex,
/// base58, base64, etc).
///
/// # Errors
///
/// Will return an error if the specified hash type is not supported.  See the docs for `Hash`
/// to see what is supported.
///
/// # Examples
///
/// ```
/// use multihash::{encode, Hash};
///
/// assert_eq!(
///     encode(Hash::SHA2256, b"hello world").unwrap(),
///     vec![18, 32, 185, 77, 39, 185, 147, 77, 62, 8, 165, 46, 82, 215, 218, 125, 171, 250, 196,
///     132, 239, 227, 122, 83, 128, 238, 144, 136, 247, 172, 226, 239, 205, 233]
/// );
/// ```
///
#[cfg(feature = "encode")]
pub fn encode(hash: Hash, input: &[u8]) -> Result<Vec<u8>, Error> {
    let size = hash.size();
    let mut output = Vec::new();
    output.resize(2 + size as usize, 0);
    output[0] = hash.code();
    output[1] = size;

    match_encoder!(hash for (input, &mut output[2..]) {
        SHA1 => ring::SHA1,
        SHA2256 => ring::SHA256,
        SHA2512 => ring::SHA512,
        SHA3224 => tiny::new_sha3_224,
        SHA3256 => tiny::new_sha3_256,
        SHA3384 => tiny::new_sha3_384,
        SHA3512 => tiny::new_sha3_512,
        Keccak224 => tiny::new_keccak224,
        Keccak256 => tiny::new_keccak256,
        Keccak384 => tiny::new_keccak384,
        Keccak512 => tiny::new_keccak512,
    });

    Ok(output)
}

/// Decodes bytes into a multihash
///
/// # Errors
///
/// Returns an error if the bytes are not a valid multihash.
///
/// # Examples
///
/// ```
/// use multihash::{decode, Hash, Multihash};
///
/// // use the data from the `encode` example
/// let data = vec![18, 32, 185, 77, 39, 185, 147, 77, 62, 8, 165, 46, 82, 215, 218,
/// 125, 171, 250, 196, 132, 239, 227, 122, 83, 128, 238, 144, 136, 247, 172, 226, 239, 205, 233];
///
/// assert_eq!(
///     decode(&data).unwrap(),
///     Multihash {
///         alg: Hash::SHA2256,
///         digest: &data[2..]
///     }
/// );
/// ```
///
pub fn decode(input: &[u8]) -> Result<Multihash, Error> {
    let code = input[0];

    let alg = Hash::from_code(code)?;
    let hash_len = alg.size() as usize;

    // length of input should be exactly hash_len + 2
    if input.len() != hash_len + 2 {
        return Err(Error::BadInputLength);
    }

    Ok(Multihash {
        alg: alg,
        digest: &input[2..],
    })
}

/// Represents a valid multihash, by associating the hash algorithm with the data
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Multihash<'a> {
    pub alg: Hash,
    pub digest: &'a [u8],
}

/// Convert bytes to a hex representation
pub fn to_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        write!(hex, "{:02x}", byte).expect("Can't fail on writing to string");
    }

    hex
}
