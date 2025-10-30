export const logs: Array<Log> = $state([]);

interface Log {
    time: string,
    level: string,
    level_style: string,
    text: string,
};

export enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace
};

const format_log = (level: LogLevel, text: string) => {
    const pad = (n: number) => String(n).padStart(2, '0');
    let now = new Date();
    let time = `[${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())}]`;

    let log: Log = {
        time,
        level: LogLevel[level].toUpperCase(),
        level_style: {
            [LogLevel.Error]: "text-red-500 font-bold",
            [LogLevel.Warn]: "text-yellow-500 font-semibold",
            [LogLevel.Info]: "text-blue-400",
            [LogLevel.Debug]: "text-green-400",
            [LogLevel.Trace]: "text-gray-400",
        }[level],
        text,
    };

    return log;
};

export const append_log = (level: LogLevel, text: string) => {
    logs.unshift(format_log(level, text));
    if (logs.length > 256) {
        logs.pop();
    }
};

