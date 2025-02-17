pub use sakura_macros::Builder;


#[test]
fn test_builder() {

    #[derive(Builder)]
    struct Service {
        #[builder(getter)]
        name: String,
        #[builder(setter)]
        count: i32,
        #[builder(getter, setter)]
        enabled: bool,
    }

    let mut service = Service::builder()
        .name("test".to_string())
        .count(42)
        .enabled(true)
        .build()
        .unwrap();

    // Use generated getter
    assert_eq!(service.get_name(), "test");

    // Use generated setter
    service = service.set_count(100);
    service = service.set_enabled(false);

}