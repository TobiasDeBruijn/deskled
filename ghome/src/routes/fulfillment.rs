use std::fmt::Debug;
use actix_web::HttpResponse;
use actix_web::web::Bytes;
use serde::{Serialize, Deserialize};
use tracing::instrument;
use crate::authorization::Auth;
use crate::data::WebData;
use crate::error::Error;
use crate::WebResult;

/// The basic request deserializing is used
/// to fish out the intent of the request.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicRequest {
    request_id: String,
    inputs: Vec<BasicInput>
}

#[derive(Debug, Deserialize)]
pub struct BasicInput {
    intent: Intent
}

#[derive(Debug, Deserialize)]
pub enum Intent {
    #[serde(rename = "action.devices.SYNC")]
    Sync,
    #[serde(rename = "action.devices.QUERY")]
    Query,
    #[serde(rename = "action.devices.EXECUTE")]
    Execute,
    #[serde(rename = "action.devices.DISCONNECT")]
    Disconnect
}

#[instrument(skip_all)]
pub async fn fulfillment(data: WebData, _: Auth, bytes: Bytes) -> WebResult<HttpResponse> {
    let bytes = bytes.to_vec();

    // We deserialize only what we need: to figure out the intent
    // We leave the deserializing of the full payload to the specific function
    // as the structure changes depending on the intent
    let basic_req: BasicRequest = serde_json::from_slice(&bytes)?;

    // While we should never enter this branch, we dont
    // want to panic if do either.
    if basic_req.inputs.is_empty() {
        return Err(Error::BadRequest);
    }

    let input = basic_req.inputs.first().unwrap();
    let ret = match input.intent {
        Intent::Sync => sync::sync(data, basic_req.request_id).await,
        Intent::Query => query::query(data, bytes).await,
        Intent::Execute => execute::execute(data, bytes).await,
        Intent::Disconnect => Ok("{}".to_string())
    }?;

    // TODO I dont like returning a string from the intent functions and then
    // manually setting the body, can this use Rust types somehow?
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(ret))
}

mod sync {
    use serde::Serialize;
    use tracing::instrument;
    use crate::data::WebData;
    use crate::routes::fulfillment::GenericResponse;
    use crate::WebResult;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct RequestPayload {
        agent_user_id: String,
        devices: Vec<Device>
    }

    #[derive(Debug, Serialize)]
    struct Device {
        id: String,
        #[serde(rename = "type")]
        device_type: String,
        traits: Vec<String>,
        name: DeviceName,
        device_info: DeviceInfo,
        attributes: DeviceAttributes
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct DeviceAttributes {
        color_model: String
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct DeviceInfo {
        manufacturer: String,
        model: String,
        hw_version: String,
        sw_version: String,
    }

    #[derive(Debug, Serialize)]
    struct DeviceName {
        name: String,
    }

    #[instrument(skip_all)]
    pub async fn sync(data: WebData, request_id: String) -> WebResult<String> {
        let payload = RequestPayload {
            devices: vec![
                Device {
                    id: "0".to_string(),
                    device_type: "action.devices.types.LIGHT".to_string(),
                    traits: vec![
                        "action.devices.traits.OnOff".to_string(),
                        "action.devices.traits.ColorSetting".to_string(),
                        "action.devices.traits.Brightness".to_string(),
                    ],
                    name: DeviceName {
                        name: "DeskLed".to_string(),
                    },
                    device_info: DeviceInfo {
                        manufacturer: "Array21 Development".to_string(),
                        model: "PiZero".to_string(),
                        hw_version: "0.1.0".to_string(),
                        sw_version: env!("CARGO_PKG_VERSION").to_string(),
                    },
                    attributes: DeviceAttributes {
                        color_model: "rgb".to_string(),
                    }
                }
            ],
            agent_user_id: data.config.login_username.clone(),
        };

        let json = serde_json::to_string(&GenericResponse {
            payload,
            request_id
        })?;

        Ok(json)
    }
}

mod query {
    use mysql::TxOpts;
    use serde::{Serialize, Deserialize};
    use tracing::instrument;
    use crate::dal::device::{get_rgb, get_state, Rgb};
    use crate::data::WebData;
    use crate::error::Error;
    use crate::routes::fulfillment::{DeviceColor, DeviceStatus, GenericRequest, GenericResponse};
    use crate::WebResult;

    #[derive(Debug, Deserialize)]
    struct RequestPayload {
        devices: Vec<QueryDevice>
    }

    #[derive(Debug, Deserialize)]
    struct QueryDevice {
        id: String,
    }

    #[derive(Debug, Serialize)]
    struct ResponsePayload {
        devices: Vec<ResponseDevice>
    }

    #[derive(Debug, Serialize)]
    struct ResponseDevice {
        #[serde(rename = "0")]
        zero: DeviceStatus
    }

    /// Returns an empty query response
    fn query_return_empty(request_id: String) -> String {
        let payload = GenericResponse {
            request_id,
            payload: ResponsePayload {
                devices: Vec::default()
            }
        };

        serde_json::to_string(&payload).unwrap()
    }

    #[instrument(skip_all)]
    pub async fn query(data: WebData, payload: Vec<u8>) -> WebResult<String> {
        let payload: GenericRequest<RequestPayload> = serde_json::from_slice(&payload)?;
        let input = payload.inputs.first().ok_or(Error::BadRequest)?;
        let device = match input.payload.devices.first() {
            Some(x) => x,
            None => return Ok(query_return_empty(payload.request_id))
        };

        if device.id.ne("0") {
            return Ok(query_return_empty(payload.request_id));
        }

        let mut tx = data.pool.start_transaction(TxOpts::default())?;
        let rgb = get_rgb(&mut tx)?.unwrap_or(Rgb::off());
        let on = get_state(&mut tx)?.unwrap_or(false);
        tx.commit()?;

        let payload = GenericResponse {
            request_id: payload.request_id,
            payload: ResponsePayload {
                devices: vec! [
                    ResponseDevice {
                        zero: DeviceStatus {
                            color: DeviceColor {
                                spectrum_rgb: rgb.clone().into_spectrum_rgb()
                            },
                            brightness: on.then(|| rgb.get_brightness()).unwrap_or(0),
                            on,
                            online: true,
                        }
                    }
                ]
            }
        };

        let ser = serde_json::to_string(&payload)?;
        Ok(ser)
    }
}

mod execute {
    use mysql::TxOpts;
    use serde::{Serialize, Deserialize};
    use tracing::instrument;
    use crate::dal::device::{get_rgb, get_state, Rgb, set_rgb, set_state};
    use crate::data::WebData;
    use crate::error::Error;
    use crate::routes::fulfillment::{DeviceColor, DeviceStatus, GenericRequest, GenericResponse};
    use crate::WebResult;

    #[derive(Debug, Deserialize)]
    struct RequestPayload {
        commands: Vec<TargettedExecution>
    }

    #[derive(Debug, Deserialize)]
    struct TargettedExecution {
        devices: Vec<ExecuteTarget>,
        execution: Vec<Command>
    }

    #[derive(Debug, Deserialize)]
    struct Command {
        command: CommandType,
        params: CommandParams,
    }

    #[derive(Debug, Deserialize)]
    enum CommandType {
        #[serde(rename = "action.devices.commands.BrightnessAbsolute")]
        BrightnessAbsolute,
        #[serde(rename = "action.devices.commands.ColorAbsolute")]
        ColorAbsolute,
        #[serde(rename = "action.devices.commands.OnOff")]
        OnOff
    }

    #[derive(Debug, Deserialize)]
    struct CommandParams {
        brightness: Option<u8>,
        color: Option<DeviceColor>,
        on: Option<bool>,
    }

    #[derive(Debug, Deserialize)]
    struct ExecuteTarget {
        id: String,
    }

    #[derive(Debug, Serialize)]
    struct ResponsePayload {
        commands: Vec<CommandResponse>,
    }

    #[derive(Debug, Serialize, Clone)]
    struct CommandResponse {
        ids: Vec<String>,
        status: String,
        states: DeviceStatus
    }

    /// Returns an empty execute response
    fn execute_empty_response(request_id: String) -> String {
        let payload = GenericResponse {
            request_id,
            payload: ResponsePayload {
                commands: vec![]
            }
        };

        serde_json::to_string(&payload).unwrap()
    }

    #[instrument(skip_all)]
    pub async fn execute(data: WebData, payload: Vec<u8>) -> WebResult<String> {
        let payload: GenericRequest<RequestPayload> = serde_json::from_slice(&payload)?;
        let input = match payload.inputs.first() {
            Some(x) => x,
            None => return Ok(execute_empty_response(payload.request_id))
        };

        let mut tx = data.pool.start_transaction(TxOpts::default())?;

        for command in &input.payload.commands {
            let has_zero = command.devices.iter()
                .filter(|x| x.id.eq("0"))
                .count() > 0;
            if !has_zero {
                return Ok(execute_empty_response(payload.request_id))
            }

            for exec in &command.execution {
                match exec.command {
                    CommandType::BrightnessAbsolute => {
                        let brightness = exec.params.brightness.ok_or(Error::BadRequest)?;

                        // Fetch the stored color and adjust its brightness
                        let mut current = get_rgb(&mut tx)?.unwrap_or(Rgb::off());
                        current.set_brightness(brightness);

                        // Apply the color to the LEDs
                        data.driver.send(current.clone()).await.expect("Channel closed");

                        // store the new color
                        set_rgb(&mut tx, current)?;
                        // Store the ON/OFF state
                        set_state(&mut tx, brightness > 0)?;
                    },
                    CommandType::ColorAbsolute => {
                        let color = exec.params.color.as_ref().ok_or(Error::BadRequest)?.spectrum_rgb;
                        let rgb = Rgb::from_spectrum_rgb(color);

                        // If the device is not turned on, we don't want to
                        // turn it on
                        let on = get_state(&mut tx)?.unwrap_or(false);
                        if on {
                            data.driver.send(rgb.clone()).await.expect("Channel closed");
                        }

                        // Store the new color
                        set_rgb(&mut tx, rgb)?;
                    },
                    CommandType::OnOff => {
                        let on = exec.params.on.ok_or(Error::BadRequest)?;

                        if on {
                            // If the user wants to turn the LEDs on,
                            // fetch the previous color, if it was black, make it white
                            let prev_state = get_rgb(&mut tx)?.unwrap_or(Rgb::on());
                            let prev_state = prev_state.is_off().then(|| Rgb::on()).unwrap_or(prev_state);

                            // set the LEDs
                            data.driver.send(prev_state.clone()).await.expect("Channel closed");
                            // store the ON/OFF state
                            set_state(&mut tx, true)?;
                            // also store the color (in case it used to be black)
                            set_rgb(&mut tx, prev_state)?;
                        } else {
                            // set the LEDs
                            data.driver.send(Rgb::off()).await.expect("Channel closed");
                            // Store the ON/OFF state
                            set_state(&mut tx, false)?;

                            // We dont set the color to black, this way
                            // when the user turns the LEDs on again,
                            // it'll restore the color/brightness they had set
                            // before they turned it off.
                        }
                    }
                }
            }
        }

        let rgb = get_rgb(&mut tx)?.unwrap_or(Rgb::off());
        let on = get_state(&mut tx)?.unwrap_or(false);

        let status = CommandResponse {
            ids: vec![
                "0".to_string()
            ],
            status: "SUCCESS".to_string(),
            states: DeviceStatus {
                brightness: on.then(|| rgb.get_brightness()).unwrap_or(0),
                on,
                online: true,
                color: DeviceColor {
                    spectrum_rgb: rgb.into_spectrum_rgb()
                }
            }
        };

        tx.commit()?;

        let payload = GenericResponse {
            request_id: payload.request_id,
            payload: ResponsePayload {
                commands: vec![status; input.payload.commands.len()]
            }
        };

        let res = serde_json::to_string(&payload)?;
        Ok(res)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenericResponse<T: Serialize + Debug> {
    request_id: String,
    payload: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenericRequest<T: Debug> {
    request_id: String,
    inputs: Vec<GenericInput<T>>
}

#[derive(Debug, Deserialize)]
struct GenericInput<T: Debug> {
    payload: T
}

#[derive(Debug, Serialize, Clone)]
struct DeviceStatus {
    on: bool,
    online: bool,
    brightness: u8,
    color: DeviceColor
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct DeviceColor {
    #[serde(rename = "spectrumRGB")]
    spectrum_rgb: i32
}