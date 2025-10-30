import { ws_send_command, type ControlState } from '$lib/ws.svelte';
import { clamp } from '$lib/utils';
import { append_log, LogLevel } from '$lib/logs.svelte';

export const keys_down = $state(new Set<string>());

export const roland_state: RolandState = $state({
    speed_multiplier: 0.9,
    left_speed_normal: 0,
    right_speed_normal: 0,
    servo_angle: 90,
    buzzer_freq: 440,
    led: { r: 0, g: 0, b: 0 },
    control_state: 'ManualControl',
    track_sensor: null,
    ultra_sensor: null,
});

export const handle_control_state = () => {
    ws_send_command({ ControlState: roland_state.control_state });
};

export const on_key_change = () => {
    handle_wasd();
};


type RolandState = {
    speed_multiplier: number,
    left_speed_normal: number,
    right_speed_normal: number,
    servo_angle: number,
    buzzer_freq: number,
    led: RGB,
    control_state: ControlState,
    track_sensor: [boolean, boolean, boolean, boolean] | null,
    ultra_sensor: number | null,
};

export type RGB = {
    r: number,
    g: number,
    b: number,
};

const handle_wasd = () => {
    let left = 0;
    let right = 0;

    if (keys_down.has('w')) {
        left++;
        right++;
    }
    if (keys_down.has('s')) {
        left--;
        right--;
    }
    if (keys_down.has('a')) {
        left--;
        right++;
    }
    if (keys_down.has('d')) {
        left++;
        right--;
    }

    left = clamp(left, -1, 1);
    right = clamp(right, -1, 1);

    if (left === roland_state.left_speed_normal && right === roland_state.right_speed_normal) {
        return;
    }

    roland_state.left_speed_normal = left;
    roland_state.right_speed_normal = right;

    left *= roland_state.speed_multiplier;
    right *= roland_state.speed_multiplier;

    ws_send_command({ Motor: [left, right] });
};

export const handle_servo = () => {
    ws_send_command({ Servo: clamp(roland_state.servo_angle - 90, -90, 90) });
};

export const handle_buzzer = (on: boolean) => {
    ws_send_command({ Buzzer: on ? roland_state.buzzer_freq : 0 });
};

let led_snapshot: [number, number, number] = [0, 0, 0];
export const handle_led = () => {
    let rgb: [number, number, number] = [roland_state.led.r, roland_state.led.g, roland_state.led.b];
    if (led_snapshot === rgb) return;
    led_snapshot = rgb;
    ws_send_command({ LED: rgb });
};

export const send_local_settings = () => {
    handle_buzzer(false);
    handle_led();
    handle_servo();

    let left = roland_state.left_speed_normal * roland_state.speed_multiplier;
    let right = roland_state.right_speed_normal * roland_state.speed_multiplier;
    ws_send_command({ Motor: [left, right] });

    handle_control_state();

    append_log(LogLevel.Trace, "Sent over local settings");
};
