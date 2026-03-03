import { error, warn, info, debug, trace } from '@tauri-apps/plugin-log';

const isTauri = '__TAURI_INTERNALS__' in window;

const logger = {
  error: (message: string) => {
    if (isTauri) {
      error(message);
    } else {
      console.error(message);
    }
  },
  warn: (message: string) => {
    if (isTauri) {
      warn(message);
    } else {
      console.warn(message);
    }
  },
  info: (message: string) => {
    if (isTauri) {
      info(message);
    } else {
      console.info(message);
    }
  },
  debug: (message: string) => {
    if (isTauri) {
      debug(message);
    } else {
      console.debug(message);
    }
  },
  trace: (message: string) => {
    if (isTauri) {
      trace(message);
    } else {
      console.log(message);
    }
  },
};

export default logger;
