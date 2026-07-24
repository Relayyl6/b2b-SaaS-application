use chrono::Utc;
use sha1::{Digest, Sha1};

pub trait StorageProvider: Send + Sync {
    fn sign_upload(
        &self,
        folder: &str,
        public_id: Option<&str>,
    ) -> Result<crate::models::SignedUploadResponse, String>;
}

#[derive(Clone)]
pub struct CloudinaryStorage {
    cloud_name: String,
    api_key: String,
    api_secret: String,
}

impl CloudinaryStorage {
    pub fn new(cloud_name: String, api_key: String, api_secret: String) -> Self {
        Self {
            cloud_name,
            api_key,
            api_secret,
        }
    }
}

impl StorageProvider for CloudinaryStorage {
    fn sign_upload(
        &self,
        folder: &str,
        public_id: Option<&str>,
    ) -> Result<crate::models::SignedUploadResponse, String> {
        let timestamp = Utc::now().timestamp();

        let mut sign_parts = vec![format!("folder={folder}"), format!("timestamp={timestamp}")];
        if let Some(pid) = public_id {
            sign_parts.push(format!("public_id={pid}"));
        }
        sign_parts.sort();
        let to_sign = format!("{}{}", sign_parts.join("&"), self.api_secret);
        let mut hasher = Sha1::new();
        hasher.update(to_sign.as_bytes());
        let signature = format!("{:x}", hasher.finalize());

        Ok(crate::models::SignedUploadResponse {
            cloud_name: self.cloud_name.clone(),
            api_key: self.api_key.clone(),
            timestamp,
            signature,
            folder: folder.to_string(),
            public_id: public_id.map(String::from),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockStorageProvider;
    impl StorageProvider for MockStorageProvider {
        fn sign_upload(
            &self,
            folder: &str,
            public_id: Option<&str>,
        ) -> Result<crate::models::SignedUploadResponse, String> {
            Ok(crate::models::SignedUploadResponse {
                cloud_name: "mock_cloud".to_string(),
                api_key: "mock_key".to_string(),
                timestamp: 1234567890,
                signature: "mock_signature".to_string(),
                folder: folder.to_string(),
                public_id: public_id.map(String::from),
            })
        }
    }

    #[test]
    fn test_cloudinary_storage_sign_upload() {
        let storage = CloudinaryStorage::new(
            "test_cloud".to_string(),
            "test_key".to_string(),
            "test_secret".to_string(),
        );

        let res = storage.sign_upload("test_folder", Some("test_id")).unwrap();
        assert_eq!(res.cloud_name, "test_cloud");
        assert_eq!(res.api_key, "test_key");
        assert_eq!(res.folder, "test_folder");
        assert_eq!(res.public_id, Some("test_id".to_string()));
        assert!(!res.signature.is_empty());
    }

    #[test]
    fn test_mock_storage_provider() {
        let mock = MockStorageProvider;
        let res = mock.sign_upload("test_folder", None).unwrap();
        assert_eq!(res.cloud_name, "mock_cloud");
        assert_eq!(res.signature, "mock_signature");
    }
}
