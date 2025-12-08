#[0xv7]/node/src/tests/rpc.test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;

    #[tokio::test]
    async fn test_get_status() {
        let client = Client::new();
        let response = client.get("http://localhost:8080/status").send().await.unwrap();
        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body.get("chain_id").is_some());
        assert!(body.get("height").is_some());
    }

    #[tokio::test]
    async fn test_get_blocks() {
        let client = Client::new();
        let response = client.get("http://localhost:8080/blocks").send().await.unwrap();
        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body.is_array());
    }

    #[tokio::test]
    async fn test_post_write() {
        let client = Client::new();
        let response = client.post("http://localhost:8080/write")
            .json(&serde_json::json!({"data": "Test transaction"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body.get("index").is_some());
        assert_eq!(body.get("data").unwrap(), "Test transaction");
    }
}