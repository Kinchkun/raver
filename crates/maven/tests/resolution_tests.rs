use httpmock::Method::GET;
use httpmock::MockServer;
use maven::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn request_example_metadata() {
    // Start a lightweight mock server.
    let server = MockServer::start();

    // Create a mock on the server.
    let hello_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/maven/de/kinch/my-artifact/maven-metadata.xml");
        then.status(200)
            .header("Content-Type", "text/xml")
            .body_from_file("tests/resources/example_metadata.xml");
    });

    let maven_repo = Repository::open_remote(
        Credentials::new("admin", "password"),
        server.url("/maven/").parse().expect("could not parse url"),
    );

    let resolved_artifact = maven_repo
        .resolve("de.kinch:my-artifact".parse().expect(""))
        .await
        .expect("Failure while resolving artifact")
        .expect("Could not find artifact");

    hello_mock.assert();
    assert_eq!(
        resolved_artifact,
        ResolvedArtifact {
            group: "de.kinch".to_string(),
            name: "my-artifact".to_string(),
            versions: vec![
                "2.0.0".to_string(),
                "2.1.0".to_string(),
                "2.2.0".to_string(),
            ]
        }
    );
}
