use anytree_sbom::Hash;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

#[derive(Serialize, Deserialize)]
enum Alg {
    MD5,
    #[serde(rename = "SHA-1")]
    SHA1,
    #[serde(rename = "SHA-256")]
    SHA256,
    #[serde(rename = "SHA-512")]
    SHA512,
}

pub fn check_hashes(hashes: &Vec<Hash>, data: impl AsRef<[u8]>) -> anyhow::Result<()> {
    for hash in hashes {
        tracing::trace!("Hash params: {:?}", hash);
        let digest = count_hash(&hash.alg, &data)?;
        tracing::trace!("Counted hash: {}", digest);
        if hash.content != digest {
            anyhow::bail!(
                "Wrong hash.\nAlg: {}\nSBOM hash: {}\nActual hash: {}",
                hash.alg,
                hash.content,
                digest
            );
        }
    }
    Ok(())
}

fn count_hash(alg: impl AsRef<str>, data: impl AsRef<[u8]>) -> anyhow::Result<String> {
    let res = match serde_json::from_str::<Alg>(&format!("\"{}\"", alg.as_ref())) {
        Ok(Alg::MD5) => md5(data),
        Ok(Alg::SHA1) => sha1(data),
        Ok(Alg::SHA256) => sha256(data),
        Ok(Alg::SHA512) => sha512(data),
        Err(_) => {
            anyhow::bail!("Unsupported algorithm: {}", alg.as_ref());
        }
    };
    Ok(hex::encode(res))
}

fn md5(data: impl AsRef<[u8]>) -> Vec<u8> {
    let mut hasher = Md5::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

fn sha1(data: impl AsRef<[u8]>) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

fn sha256(data: impl AsRef<[u8]>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

fn sha512(data: impl AsRef<[u8]>) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}
