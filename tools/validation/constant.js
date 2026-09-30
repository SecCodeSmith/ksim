// Constant arrival rate for calibration: RATE requests/s against URL for DURATION.
import http from 'k6/http';

export const options = {
  scenarios: {
    calibrate: {
      executor: 'constant-arrival-rate',
      rate: Number(__ENV.RATE),
      timeUnit: '1s',
      duration: __ENV.DURATION,
      preAllocatedVUs: 20,
      maxVUs: 100,
    },
  },
};

export default function () {
  http.get(__ENV.URL);
}
