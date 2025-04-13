use super::helpers::TestApp;

#[tokio::test]
async fn login_returns_200_for_valid_credentials() {
    let app = TestApp::new().await;

    let response = app.login("test@example.com", "password123").await;

    assert_eq!(response.status().as_u16(), 200);
}

/*
#[tokio::test]
async fn login_returns_401_for_invalid_credentials() {
    let app = TestApp::new().await;

    let response = app.login("invalid-email", "wrong-password").await;

    assert_eq!(response.status().as_u16(), 401);
}
 */
