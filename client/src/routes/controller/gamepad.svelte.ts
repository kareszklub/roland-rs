let poll: number | null = null;

const buttons = ["a", "b", "x", "y", "lb", "rb", "lt", "rt", "map", "menu", "lstick", "rstick", "du", "dd", "dl", "dr", "xbox"] as const;
const axes = ["lx", "ly", "rx", "ry"] as const;

type ButtonName = typeof buttons[number];
type AxisName = typeof axes[number];

export const button_map = $state(Object.fromEntries(buttons.map(b => [b, 0])) as Record<ButtonName, number>);
export const axis_map = $state(Object.fromEntries(axes.map(a => [a, 0])) as Record<AxisName, number>);

const start_controller = () => {
    const gamepads = navigator.getGamepads();
    if (gamepads.length === 0 || !gamepads[0]) return;
    const pad = gamepads[0];

    pad.buttons.forEach((button, i) => {
        button_map[buttons[i]] = (button.pressed) ? button.value : 0;
    });

    pad.axes.forEach((axis, i) => {
        axis_map[axes[i]] = (axis > 0.01 || axis < -0.01) ? parseFloat(axis.toFixed(3)) : 0;
    });

    poll = requestAnimationFrame(start_controller);
};

export const handle_gamepad_connect = () => { start_controller(); };
export const handle_gamepad_disconnect = () => {
    if (poll) {
        cancelAnimationFrame(poll);
    }
};
