import { send_local_settings } from "../routes/controller/controller.svelte";
import { append_log, LogLevel } from "./logs.svelte";

export let roland = $state({ ip: "10.115.123.4", connection: "disconnected" });

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

export type Command = BuzzerCommand | MotorCommand | ServoCommand | LEDCommand;

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
