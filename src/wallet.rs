use p256::ecdsa::{SigningKey, VerifyingKey};
use p256::ecdsa::Signature;
use rand_core::OsRng;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::ser::SerializeStruct;
use serde::de::Error as DeError;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{self};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub public_key: VerifyingKey,
    pub private_key: SigningKey,
}

impl Wallet {
    pub fn new() -> Self {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = VerifyingKey::from(&private_key);
        Self { private_key, public_key }
    }

    pub fn get_address(&self) -> String {
        // Simplified address generation for demonstration purposes
        format!("{:?}", self.public_key.to_encoded_point(false))
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let file = File::create(path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let wallet = serde_json::from_reader(file)?;
        Ok(wallet)
    }
}

impl Serialize for Wallet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Wallet", 2)?;
        state.serialize_field("public_key", &self.public_key.to_encoded_point(false).as_bytes())?;
        state.serialize_field("private_key", &self.private_key.to_bytes().as_slice())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Wallet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct WalletData {
            public_key: Vec<u8>,
            private_key: Vec<u8>,
        }

        let data = WalletData::deserialize(deserializer)?;
        let encoded_point = p256::EncodedPoint::from_bytes(&data.public_key).map_err(DeError::custom)?;
        let public_key = VerifyingKey::from_encoded_point(&encoded_point).map_err(DeError::custom)?;
        let private_key = SigningKey::try_from(&data.private_key[..]).map_err(DeError::custom)?;
        Ok(Wallet { public_key, private_key })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptionalSerializableSignature(pub Option<SerializableSignature>);

#[derive(Debug, Clone)]
pub struct SerializableSignature(pub Signature);

impl Serialize for SerializableSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let der_encoded = self.0.to_der();
        let bytes = der_encoded.as_bytes();
        serializer.serialize_bytes(bytes)
    }
}

impl<'de> Deserialize<'de> for SerializableSignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        Signature::from_der(&bytes).map(SerializableSignature).map_err(DeError::custom)
    }
}