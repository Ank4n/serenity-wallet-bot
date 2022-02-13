extern crate google_sheets4 as sheets4;
extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;
use sheets4::api::ValueRange;
use sheets4::Error;
use sheets4::Sheets;
use std::default::Default;

async fn something() {
    let secret = yup_oauth2::read_application_secret("google_credentials.json")
        .await
        .expect("client secret could not be read");

    let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
        secret,
        yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .build()
    .await
    .unwrap();

    let mut hub = Sheets::new(
        hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
        auth,
    );

    let mut req = ValueRange::default();

    let result = hub
        .spreadsheets()
        .values_append(req, "spreadsheetId", "range")
        .value_input_option("no")
        .response_value_render_option("ipsum")
        .response_date_time_render_option("voluptua.")
        .insert_data_option("At")
        .include_values_in_response(false)
        .doit()
        .await;

    match result {
        Err(e) => match e {
            Error::HttpError(_)
            | Error::Io(_)
            | Error::MissingAPIKey
            | Error::MissingToken(_)
            | Error::Cancelled
            | Error::UploadSizeLimitExceeded(_, _)
            | Error::Failure(_)
            | Error::BadRequest(_)
            | Error::FieldClash(_)
            | Error::JsonDecodeError(_, _) => println!("{}", e),
        },
        Ok(res) => println!("Success: {:?}", res),
    }
}
pub fn save(
    username: &String,
    user_id: &String,
    wallet_type: &String,
    wallet_address: &String,
) -> Option<Error> {
    None
}
