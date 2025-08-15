use anyhow::anyhow;
use log::{error, info};
use postcard::{from_bytes, to_stdvec};
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf, split},
    sync::{broadcast, mpsc},
};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

/// pico -> pi
#[derive(Deserialize, Debug, Clone)]
pub enum SerialData {
    UltraSensor(u8),
}

/// pi -> pico
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
}

async fn find_pico_path() -> anyhow::Result<String> {
    let mut entries = fs::read_dir("/dev").await?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(name) = entry.file_name().to_str() {
            if name.starts_with("ttyACM") {
                return Ok(format!("/dev/{}", name));
            }
        }
    }

    Err(anyhow!("Pico not found"))
}

pub async fn init(
    cmd_rx: mpsc::Receiver<SerialCMD>,
    data_tx: broadcast::Sender<SerialData>,
) -> anyhow::Result<()> {
    let path = find_pico_path().await?;
    let port = tokio_serial::new(&path, 115200).open_native_async()?;

    info!("TTY-ACM port opened on {}", path);

    let (reader, writer) = split(port);

    tokio::spawn(async move {
        read_task(reader, data_tx).await;
    });

    tokio::spawn(async move {
        write_task(writer, cmd_rx).await;
    });

    Ok(())
}

async fn read_task(mut reader: ReadHalf<SerialStream>, data_tx: broadcast::Sender<SerialData>) {
    let mut buf = vec![0u8; 64];
    loop {
        match reader.read(&mut buf).await {
            Ok(0) => {
                error!("Serial port closed by peer");
                break;
            }
            Ok(n) => match from_bytes::<SerialData>(&buf[..n]) {
                Ok(data) => {
                    if let Err(e) = data_tx.send(data.clone()) {
                        error!("Couldn't send data: {}", e);
                    } else {
                        info!("Received: {:?}", data)
                    }
                }
                Err(e) => error!("Couldn't parse data({:?}): {}", &buf[..n], e),
            },
            Err(e) => {
                error!("Read error: {}", e);
                break;
            }
        }
    }
}

async fn write_task(mut writer: WriteHalf<SerialStream>, mut cmd_rx: mpsc::Receiver<SerialCMD>) {
    while let Some(cmd) = cmd_rx.recv().await {
        let data = to_stdvec(&cmd).unwrap();
        if let Err(e) = writer.write_all(&data).await {
            error!("Write error: {}", e);
        } else {
            info!("Sent: {:?}", cmd);
        }
    }
}
