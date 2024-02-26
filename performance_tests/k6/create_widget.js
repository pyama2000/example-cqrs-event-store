import http from "k6/http";
import { sleep } from "k6";

export const options = {
  scenarios: {
    shared: {
      executor: "shared-iterations",
      vus: 10,
      iterations: 200,
      maxDuration: "60s",
    },
    ramp_up: {
      executor: "ramping-vus",
      startVUs: 1,
      stages: [
        { duration: "10s", target: 10 },
        { duration: "10s", target: 20 },
        { duration: "30s", target: 50 },
        { duration: "60s", target: 100 },
      ],
    },
  },
};

export default function () {
  const url = "http://localhost:8080/widgets";
  const payload = JSON.stringify({
    widget_name: "部品名",
    widget_description: "部品の説明",
  });
  const params = {
    headers: {
      "Content-Type": "application/json",
    },
  };
  http.post(url, payload, params);
  sleep(1);
}
