import { roland_state, send_local_settings } from "../routes/controller/controller.svelte";
import { append_log, LogLevel } from "./logs.svelte";

export let roland = $state({ ip: "10.93.154.4", connection: "disconnected" });

type BuzzerCommand = {
    Buzzer: number;
};

type LEDCommand = {
    LED: [number, number, number];
};

type ServoCommand = {
    Servo: number;
};

type MotorCommand = {
    Motor: [number, number];
};

type StateCommand = {
    ControlState: ControlState;
};

export type ControlState = "ManualControl" | "FollowLine" | "KeepDistance";

export type Command = BuzzerCommand | MotorCommand | ServoCommand | LEDCommand | StateCommand;

type TextMessage = {
    Text: String,
};

type UltraSensorMessage = {
    Ultra: number | null;
};

type TrackSensorMessage = {
    Track: [boolean, boolean, boolean, boolean];
};

export type ServerMessage = TextMessage | UltraSensorMessage | TrackSensorMessage;

let ws: WebSocket | null = null;

export const ws_connect = () => {
    roland.connection = "connecting";
    ws = new WebSocket(`ws://${roland.ip}:9001`)

    ws.onopen = () => {
        roland.connection = "connected";
        append_log(LogLevel.Debug, "Connected via WebSocket");
        send_local_settings();
    };

    ws.onclose = () => {
        roland.connection = "disconnected";
        append_log(LogLevel.Debug, "WebSocket disconnected");
    };

    ws.onerror = (e) => {
        roland.connection = "disconnected";
        append_log(LogLevel.Warn, `WebSocket error: ${JSON.stringify(e)}`);
    };

    ws.onmessage = (e) => {
        ws_handle_message(JSON.parse(e.data));
        append_log(LogLevel.Info, `Received: ${e.data}`);
    };
};

export const ws_disconnect = () => {
    if (!ws) return;
    ws.close();
};

export const ws_send_command = (json: Command) => {
    if (!ws) return;
    ws.send(JSON.stringify(json));
    append_log(LogLevel.Trace, `Sent: ${JSON.stringify(json)}`);
}

export const ws_handle_message = (msg: ServerMessage) => {
    if ("Text" in msg) {
        append_log(LogLevel.Info, `Received: ${msg.Text}`);
    } else if ("Ultra" in msg) {
        if (msg.Ultra !== null) {
            roland_state.ultra_sensor = msg.Ultra;
        }
    } else if ("Track" in msg) {
        roland_state.track_sensor = msg.Track;
    } else {
        const _exhaustive: never = msg;
        append_log(LogLevel.Error, `Unknown message type: ${_exhaustive}`);
    }
};
