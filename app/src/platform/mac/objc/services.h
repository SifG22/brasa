// Our class for handling NSServices messages.
@interface BrasaServicesProvider : NSObject
@end

// Functions implemented in Rust.
id brasa_services_provider_custom_url_scheme();
void brasa_app_open_urls(id app, id urls);
