use std::fmt::Debug;
use std::str::Chars;
use actix_web::HttpResponse;
use actix_web::web::Bytes;
use mysql::TxOpts;
use serde::{Serialize, Deserialize};
use tracing::{info, instrument};
use crate::authorization::Auth;
use crate::dal::device::{get_rgb, Rgb, set_rgb};
use crate::data::WebData;
use crate::error::Error;
use crate::WebResult;

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
    let basic_req: BasicRequest = serde_json::from_slice(&bytes)?;

    if basic_req.inputs.is_empty() {
        return Err(Error::BadRequest);
    }

    let input = basic_req.inputs.first().unwrap();
    let ret = match input.intent {
        Intent::Sync => sync(data, basic_req.request_id).await,
        Intent::Query => query(data, bytes).await,
        Intent::Execute => execute(data, bytes).await,
        Intent::Disconnect => Ok("{}".to_string())
    }?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(ret))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenericResponse<T: Serialize + Debug> {
    request_id: String,
    payload: T,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncPayload {
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
async fn sync(data: WebData, request_id: String) -> WebResult<String> {
    let payload = SyncPayload {
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

#[derive(Debug, Deserialize)]
struct QueryRequestPayload {
    devices: Vec<QueryDevice>
}

#[derive(Debug, Deserialize)]
struct QueryDevice {
    id: String,
}

#[derive(Debug, Serialize)]
struct QueryResponsePayload {
    devices: Vec<QueryResponseDevice>
}

#[derive(Debug, Serialize)]
struct QueryResponseDevice {
    #[serde(rename = "0")]
    zero: DeviceStatus
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

fn query_return_empty(request_id: String) -> String {
    let payload = GenericResponse {
        request_id,
        payload: QueryResponsePayload {
            devices: Vec::default()
        }
    };

    serde_json::to_string(&payload).unwrap()
}

fn rgb_to_spectrum_rgb(rgb: Rgb) -> i32 {
    let hex = format!("{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b);
    i32::from_str_radix(&hex, 16).unwrap()
}

fn spectrum_rgb_to_rgb(spectrum_rgb: i32) -> Rgb {
    let hex = format!("{:06X}", spectrum_rgb);
    let mut chars = hex.chars();

    let color = |chars: &mut Chars| {
        let hex = format!("{}{}", chars.nth(0).unwrap(), chars.nth(0).unwrap());
        u8::from_str_radix(&hex, 16).unwrap()
    };

    Rgb {
        r: color(&mut chars),
        g: color(&mut chars),
        b: color(&mut chars),
    }
}

#[cfg(test)]
mod test {
    use crate::dal::device::Rgb;
    use super::{rgb_to_spectrum_rgb, spectrum_rgb_to_rgb};

    #[test]
    fn test_rgb_to_spectrum_rgb() {
        assert_eq!(16711935, rgb_to_spectrum_rgb(Rgb { r: 255, g: 0, b: 255 }));
    }

    #[test]
    fn test_spectrum_rgb_to_rgb() {
        assert_eq!(Rgb { r: 255, g: 0, b: 255 }, spectrum_rgb_to_rgb(16711935))
    }
}

#[instrument(skip_all)]
async fn query(data: WebData, payload: Vec<u8>) -> WebResult<String> {
    let payload: GenericRequest<QueryRequestPayload> = serde_json::from_slice(&payload)?;
    let input = payload.inputs.first().ok_or(Error::BadRequest)?;
    let device = match input.payload.devices.first() {
        Some(x) => x,
        None => return Ok(query_return_empty(payload.request_id))
    };

    if device.id.ne("0") {
        return Ok(query_return_empty(payload.request_id));
    }

    let mut tx = data.pool.start_transaction(TxOpts::default())?;
    let rgb = get_rgb(&mut tx)?.unwrap_or(Rgb { r: 0, g: 0, b: 0 });

    let payload = GenericResponse {
        request_id: payload.request_id,
        payload: QueryResponsePayload {
            devices: vec! [
                QueryResponseDevice {
                    zero: DeviceStatus {
                        color: DeviceColor {
                            spectrum_rgb: rgb_to_spectrum_rgb(rgb.clone())
                        },
                        brightness: rgb.get_brightness(),
                        on: rgb.r > 0 || rgb.g > 0 || rgb.b > 0,
                        online: true,
                    }
                }
            ]
        }
    };

    let ser = serde_json::to_string(&payload)?;
    Ok(ser)
}

#[derive(Debug, Deserialize)]
struct ExecuteRequestPayload {
    commands: Vec<ExecuteCommand>
}

#[derive(Debug, Deserialize)]
struct ExecuteCommand {
    devices: Vec<ExecuteDevice>,
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
struct ExecuteDevice {
    id: String,
}

#[derive(Debug, Serialize)]
struct ExecuteResponsePayload {
    commands: Vec<ExecuteResponseCommand>,
}

#[derive(Debug, Serialize, Clone)]
struct ExecuteResponseCommand {
    ids: Vec<String>,
    status: String,
    states: DeviceStatus
}

fn execute_empty_response(request_id: String) -> String {
    let payload = GenericResponse {
        request_id,
        payload: ExecuteResponsePayload {
            commands: vec![]
        }
    };

    serde_json::to_string(&payload).unwrap()
}

#[instrument(skip_all)]
async fn execute(data: WebData, payload: Vec<u8>) -> WebResult<String> {
    let payload: GenericRequest<ExecuteRequestPayload> = serde_json::from_slice(&payload)?;
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
                    info!("Brightness Absolute");

                    let brightness = exec.params.brightness.ok_or(Error::BadRequest)?;
                    let mut current = get_rgb(&mut tx)?.unwrap_or(Rgb { r: 0, g: 0, b: 0 });
                    current.set_brightness(brightness);

                    data.driver.send(current.clone()).await.expect("Channel closed");
                    set_rgb(&mut tx, current)?;
                },
                CommandType::ColorAbsolute => {
                    info!("Color Absolute");

                    let color = exec.params.color.as_ref().ok_or(Error::BadRequest)?.spectrum_rgb;
                    let rgb = spectrum_rgb_to_rgb(color);
                    data.driver.send(rgb.clone()).await.expect("Channel closed");
                    set_rgb(&mut tx, rgb)?;
                },
                CommandType::OnOff => {
                    info!("On Off");

                    let on = exec.params.on.ok_or(Error::BadRequest)?;

                    if on {
                        let prev_state = get_rgb(&mut tx)?.unwrap_or(Rgb { r: 255, g: 255, b: 255 });
                        data.driver.send(prev_state.clone()).await.expect("Channel closed");
                    } else {
                        data.driver.send(Rgb { r: 0, b: 0, g: 0 }).await.expect("Channel closed");
                        set_rgb(&mut tx, Rgb { r: 0, b: 0, g: 0 })?;
                    }
                }
            }
        }
    }

    let rgb = get_rgb(&mut tx)?.unwrap_or(Rgb { r: 0, g: 0, b: 0 });
    let status = ExecuteResponseCommand {
        ids: vec![
            "0".to_string()
        ],
        status: "SUCCESS".to_string(),
        states: DeviceStatus {
            brightness: rgb.get_brightness(),
            on: rgb.r > 0 || rgb.g > 0 || rgb.b > 0,
            online: true,
            color: DeviceColor {
                spectrum_rgb: rgb_to_spectrum_rgb(rgb)
            }
        }
    };

    tx.commit()?;

    let payload = GenericResponse {
        request_id: payload.request_id,
        payload: ExecuteResponsePayload {
            commands: vec![status; input.payload.commands.len()]
        }
    };

    let res = serde_json::to_string(&payload)?;
    Ok(res)
}