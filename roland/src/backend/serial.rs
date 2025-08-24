use anyhow::anyhow;
use log::{debug, error, info, trace};
use postcard::{from_bytes, to_stdvec};
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf, split},
    sync::{broadcast, mpsc},
};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::sync::CancellationToken;

use crate::backend::pico::Pico;

#[derive(Deserialize, Debug, Clone)]
pub enum TrackSensorID {
    L1,
    L2,
    R1,
    R2,
}

/// data packet coming from the pico
/// currently it's only used for sensor data
#[derive(Deserialize, Debug, Clone)]
pub enum SerialData {
    /// measured distance in cm
    UltraSensor(Option<u16>),
    /// sensor id and value
    TrackSensor((TrackSensorID, bool)),
}

/// command packet for direct control of devices managed by the pico
#[derive(Serialize, Debug)]
pub enum SerialCMD {
    /// frequency (Hz)
    Buzzer(u16),
    /// RGB color (0 to 255)
    LED((u8, u8, u8)),
    /// rotation in degrees (0 is the midpoint, -90 to 90)
    Servo(i8),
    /// duty cycle (sign is direction)
    HBridge((i32, i32)),
    /// this variant only exists on the pi side
    /// upon receiving this message, all commands defined listed in it will get sent, and the serial is
    /// closed
    Reset(Vec<SerialCMD>),
}

/// try finding the pico device
/// currently it returns the path for the first device named ttyACM*
/// TODO: make this actually verify that the device is a pico
async fn find_pico_path() -> anyhow::Result<String> {
    let mut entries = fs::read_dir("/dev").await?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(name) = entry.file_name().to_str()
            && name.starts_with("ttyACM")
        {
            return Ok(format!("/dev/{}", name));
        }
    }

    Err(anyhow!("Pico not found"))
}

/// initialize serial communication with the Pico
/// returns a clone-able Pico device
pub async fn init(token: CancellationToken) -> anyhow::Result<Pico> {
    let (cmd_tx, cmd_rx) = mpsc::channel::<SerialCMD>(32);
    let (data_tx, data_rx) = broadcast::channel::<SerialData>(32);

    let path = find_pico_path().await?;
    let port = tokio_serial::new(&path, 115200).open_native_async()?;

    info!("TTY-ACM port opened on {}", path);

    let (reader, writer) = split(port);

    {
        let token = token.clone();
        tokio::spawn(async move {
            tokio::select! {
                ret = read_task(reader, data_tx) => {
                    match ret {
                        Ok(()) => debug!("[Serial Data] task shutting down"),
                        Err(e) => error!("[Serial Data] task shutting down: {}", e),
                    }
                    token.cancel();
                },
                ret = write_task(writer, cmd_rx) => {
                    match ret {
                        Ok(()) => debug!("[Serial Write] task shutting down"),
                        Err(e) => error!("[Serial Write] task shutting down: {}", e),
                    }
                    token.cancel();
                }
                _ = token.cancelled() => {},
            }
        });
    }

    Ok(Pico::new(cmd_tx, data_rx, token))
}

/// reads and deserializes all incoming traffic, forwarding it to the data channel
async fn read_task(
    mut reader: ReadHalf<SerialStream>,
    data_tx: broadcast::Sender<SerialData>,
) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 64];
    loop {
        match reader.read(&mut buf).await? {
            0 => {
                return Err(anyhow!("Serial port closed by peer"));
            }
            n => match from_bytes::<SerialData>(&buf[..n]) {
                Ok(data) => {
                    if let Err(e) = data_tx.send(data.clone()) {
                        error!("Couldn't send data: {}", e);
                    } else {
                        trace!("Received: {:?}", data)
                    }
                }
                Err(e) => error!("Couldn't parse data({:?}): {}", &buf[..n], e),
            },
        }
    }
}

/// serializes commands going to the pico
async fn write_task(
    mut writer: WriteHalf<SerialStream>,
    mut cmd_rx: mpsc::Receiver<SerialCMD>,
) -> anyhow::Result<()> {
    while let Some(cmd) = cmd_rx.recv().await {
        if let SerialCMD::Reset(cmds) = cmd {
            for cmd in cmds {
                let data = to_stdvec(&cmd).unwrap();
                writer.write_all(&data).await?;
            }
            break;
        }

        let data = to_stdvec(&cmd).unwrap();
        writer.write_all(&data).await?;
        trace!("Sent: {:?}", cmd);
    }
    Ok(())
}
